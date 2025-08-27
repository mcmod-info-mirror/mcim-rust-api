use bson::doc;
use futures::stream::TryStreamExt;
use mongodb::{bson::Document, Client as Mongo_Client};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::database::get_database_name;
use crate::errors::ServiceError;
use crate::models::modrinth::entities as db;
use crate::models::modrinth::responses::*;

pub struct ModrinthService {
    db: Mongo_Client,
    redis: Arc<MultiplexedConnection>,
}

impl ModrinthService {
    pub fn new(db: Mongo_Client, redis: Arc<MultiplexedConnection>) -> Self {
        Self { db, redis }
    }

    // 缓存 project_id <-> slug 映射
    async fn cache_project_mapping(
        &self,
        project_id: &str,
        slug: &str,
    ) -> Result<(), ServiceError> {
        let mut conn = self.redis.as_ref().clone();
        let ttl = 3600; // 1小时过期

        // 设置双向映射
        let project_key = format!("modrinth:project_id:{}", project_id);
        let slug_key = format!("modrinth:slug:{}", slug);

        if let Err(e) = conn
            .set_ex::<String, String, ()>(project_key, slug.to_string(), ttl)
            .await
        {
            log::warn!("Failed to cache project_id->slug mapping: {}", e);
        }

        if let Err(e) = conn
            .set_ex::<String, String, ()>(slug_key, project_id.to_string(), ttl)
            .await
        {
            log::warn!("Failed to cache slug->project_id mapping: {}", e);
        }

        Ok(())
    }

    // 获取缓存的 project_id 通过 slug
    async fn get_cached_project_id(&self, slug: &str) -> Option<String> {
        let mut conn = self.redis.as_ref().clone();
        let key = format!("modrinth:slug:{}", slug);

        match conn.get::<String, Option<String>>(key).await {
            Ok(result) => result,
            Err(e) => {
                log::warn!("Failed to get cached project_id for slug {}: {}", slug, e);
                None
            }
        }
    }

    // 获取缓存的 slug 通过 project_id
    async fn get_cached_slug(&self, project_id: &str) -> Option<String> {
        let mut conn = self.redis.as_ref().clone();
        let key = format!("modrinth:project_id:{}", project_id);

        match conn.get::<String, Option<String>>(key).await {
            Ok(result) => result,
            Err(e) => {
                log::warn!(
                    "Failed to get cached slug for project_id {}: {}",
                    project_id,
                    e
                );
                None
            }
        }
    }

    // 缓存 {algorithm: hash} <-> version_id 映射
    async fn cache_hash_version_mapping(
        &self,
        algorithm: &str,
        hash: &str,
        version_id: &str,
    ) -> Result<(), ServiceError> {
        let mut conn = self.redis.as_ref().clone();
        let ttl = 3600; // 1小时过期

        let hash_key = format!("modrinth:{}:{}", algorithm, hash);

        if let Err(e) = conn
            .set_ex::<String, String, ()>(hash_key, version_id.to_string(), ttl)
            .await
        {
            log::warn!(
                "Failed to cache {}:{} -> version_id mapping: {}",
                algorithm,
                hash,
                e
            );
        }

        Ok(())
    }

    // 获取缓存的 version_id 通过 hash
    async fn get_cached_version_id(&self, algorithm: &str, hash: &str) -> Option<String> {
        let mut conn = self.redis.as_ref().clone();
        let key = format!("modrinth:{}:{}", algorithm, hash);

        match conn.get::<String, Option<String>>(key).await {
            Ok(result) => result,
            Err(e) => {
                log::warn!(
                    "Failed to get cached version_id for {}:{}: {}",
                    algorithm,
                    hash,
                    e
                );
                None
            }
        }
    }

    async fn add_project_ids_into_queue(
        &self,
        project_ids: Vec<String>,
    ) -> Result<(), ServiceError> {
        if project_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();
        conn
            .sadd::<&str, &Vec<String>, ()>("modrinth_project_ids", &project_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add project ids to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added project ids to Redis queue: {:?}", project_ids);
        Ok(())
    }

    async fn add_version_ids_into_queue(
        &self,
        version_ids: Vec<String>,
    ) -> Result<(), ServiceError> {
        if version_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();
        conn
            .sadd::<&str, &Vec<String>, ()>("modrinth_version_ids", &version_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add version ids to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added version ids to Redis queue: {:?}", version_ids);
        Ok(())
    }

    async fn add_hashes_into_queue(
        &self,
        algorithm: String,
        hashes: Vec<String>,
    ) -> Result<(), ServiceError> {
        if hashes.is_empty() {
            return Ok(()); // 如果 hash 为空，直接返回
        }

        let mut conn = self.redis.as_ref().clone();

        conn
            .sadd::<&str, &Vec<String>, ()>(
                format!("modrinth_hashes_{}", algorithm).as_str(),
                &hashes,
            )
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add hash to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added {}:{} to Redis queue", algorithm, hashes.join(","));
        Ok(())
    }

    async fn check_search_result(&self, data: &serde_json::Value) -> Result<(), ServiceError> {
        if data.is_null() || !data.is_object() {
            return Err(ServiceError::UnexpectedError(
                "Search result is null".to_string(),
            ));
        }

        let projects = data
            .get("hits")
            .ok_or_else(|| {
                ServiceError::UnexpectedError("No 'hits' field in response".to_string())
            })?
            .as_array()
            .ok_or_else(|| ServiceError::UnexpectedError("'hits' is not an array".to_string()))?;

        let mut project_ids = Vec::new();

        for _project in projects {
            if let Some(project_id) = _project
                .as_object()
                .and_then(|obj| obj.get("project_id"))
                .and_then(|v| v.as_str())
            {
                project_ids.push(project_id.to_string());
            } else {
                return Err(ServiceError::UnexpectedError(
                    "Project ID not found in search result".to_string(),
                ));
            }
        }

        if project_ids.is_empty() {
            log::debug!("Search result is empty, no project IDs found");
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<bson::Document>("modrinth_projects");

        let mut cursor = collection
            .find(doc! { "_id": { "$in": &project_ids } })
            .projection(doc! { "_id": 1 })
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch project documents: {}", e),
                source: Some(e),
            })?;

        let mut found_project_ids = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch project documents: {}", e),
                source: Some(e),
            })?
        {
            if let Ok(id) = doc.get_str("_id") {
                found_project_ids.push(id.to_string());
            }
        }

        let not_found_project_ids: Vec<String> = project_ids
            .iter()
            .filter(|id| !found_project_ids.contains(id))
            .cloned()
            .collect();

        if !not_found_project_ids.is_empty() {
            self.add_project_ids_into_queue(not_found_project_ids)
                .await?;
        } else {
            log::trace!("All projects have been found in the database.");
        }

        Ok(())
    }

    pub async fn search(
        &self,
        client: &Client,
        query: Option<String>,
        facets: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
        index: Option<String>,
        modrinth_api_url: &str,
    ) -> Result<serde_json::Value, ServiceError> {
        let api_url = format!("{}/v2/search", modrinth_api_url);

        // 验证 index 在 SearchIndex 内
        if let Some(ref idx) = index {
            if !["relevance", "downloads", "follows", "newest", "updated"].contains(&idx.as_str()) {
                return Err(ServiceError::InvalidInput {
                    field: String::from("index"),
                    reason: format!("Invalid index value: {}", idx),
                });
            }
        }

        let mut params = vec![];
        if let Some(q) = query {
            params.push(("query", q));
        }
        if let Some(f) = facets {
            params.push(("facets", f));
        }
        if let Some(o) = offset {
            params.push(("offset", o.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(i) = index {
            params.push(("index", i.to_string()));
        }

        let response = client
            .get(api_url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError {
                service: String::from("Modrinth API"),
                message: format!("Failed to send request: {}", e),
            })?;

        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| ServiceError::ExternalServiceError {
                service: String::from("Modrinth API"),
                message: format!("Failed to read response body: {}", e),
            })?;
        let search_result = serde_json::from_slice(&bytes).map_err(|e| {
            ServiceError::UnexpectedError(format!(
                "Failed to parse JSON: {}, text: {}",
                e,
                String::from_utf8_lossy(&bytes)
            ))
        })?;

        if status.is_success() {
            // 检查有无未缓存的 Project
            match self.check_search_result(&search_result).await {
                Ok(_) => log::trace!("Search result check completed successfully"),
                Err(e) => log::error!("Modrinth Search result check failed: {}", e),
            };
        }

        Ok(search_result)
    }

    pub async fn get_project_by_id_or_slug(
        &self,
        project_id_or_slug: String,
    ) -> Result<Option<Project>, ServiceError> {
        if project_id_or_slug.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("project_id or slug"),
                reason: String::from("Project_id or slug cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::Project>("modrinth_projects");

        match collection
            .find_one(doc! { "$or": [
                { "_id": &project_id_or_slug },
                { "slug": &project_id_or_slug }
            ] })
            .await?
        {
            Some(doc) => {
                // 缓存项目映射关系以供未来使用
                if let Err(e) = self.cache_project_mapping(&doc.id, &doc.slug).await {
                    log::warn!("Failed to cache project mapping: {}", e);
                }

                Ok(Some(doc.into()))
            }
            // None => Err(ServiceError::NotFound {
            //     resource: String::from("Modrinth Project"),
            //     detail: Some(format!(
            //         "Project with project_id or slug {} not found",
            //         project_id_or_slug
            //     )),
            None => {
                self.add_project_ids_into_queue(vec![project_id_or_slug.clone()])
                    .await?;
                Err(ServiceError::NotFound {
                    resource: String::from("Modrinth Project"),
                    detail: Some(format!(
                        "Project with project_id or slug {} not found",
                        project_id_or_slug
                    )),
                })
            }
        }
    }

    pub async fn get_projects(
        &self,
        project_ids_or_slugs: Vec<String>,
    ) -> Result<Vec<Project>, ServiceError> {
        if project_ids_or_slugs.is_empty() {
            return Ok(Vec::new()); // 官方返回的是 []
        }

        // 优化: 先从Redis缓存中查找slug->project_id映射
        let mut resolved_project_ids = Vec::new();
        let mut remaining_items = Vec::new();

        for item in project_ids_or_slugs.clone() {
            // 判断是否为project_id格式 (通常是8个字符的字母数字组合)
            if item.len() == 8 && item.chars().all(|c| c.is_ascii_alphanumeric()) {
                resolved_project_ids.push(item);
            } else {
                // 可能是slug，尝试从缓存获取对应的project_id
                if let Some(cached_project_id) = self.get_cached_project_id(&item).await {
                    log::trace!(
                        "Found cached project_id {} for slug {}",
                        cached_project_id,
                        item
                    );
                    resolved_project_ids.push(cached_project_id);
                } else {
                    // 缓存中没找到，加入待查询列表
                    remaining_items.push(item);
                }
            }
        }

        // 如果所有项目都从缓存中找到了对应的project_id，直接按project_id查询
        if remaining_items.is_empty() && !resolved_project_ids.is_empty() {
            log::trace!(
                "All items resolved from cache, querying {} project_ids directly",
                resolved_project_ids.len()
            );
        }

        // 否则还需要查询数据库
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::Project>("modrinth_projects");

        // 构建查询条件：包含已解析的project_id和剩余的待查询项
        let mut all_items_to_query = resolved_project_ids;
        all_items_to_query.extend(remaining_items.clone());

        let filter = doc! {
            "$or": [
                { "_id": { "$in": &all_items_to_query.clone() } },
                { "slug": { "$in": &all_items_to_query } }
            ]
        };

        let mut cursor =
            collection
                .find(filter)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: format!("Failed to fetch project documents: {}", e),
                    source: Some(e),
                })?;

        let mut projects: Vec<Project> = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch project documents: {}", e),
                source: Some(e),
            })?
        {
            // 缓存新发现的项目映射关系
            if let Err(e) = self.cache_project_mapping(&doc.id, &doc.slug).await {
                log::warn!("Failed to cache project mapping for {}: {}", doc.id, e);
            }

            projects.push(doc.into());
        }

        // 将未找到的 Project 添加到队列
        if projects.is_empty() {
            self.add_project_ids_into_queue(project_ids_or_slugs.clone())
                .await?;
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth Project"),
                detail: Some(format!(
                    "No projects found for the provided project_ids or slugs: {}",
                    project_ids_or_slugs.join(",")
                )),
            });
        } else {
            let not_found_items: Vec<String> = all_items_to_query
                .iter()
                .filter(|item| {
                    !projects
                        .iter()
                        .any(|p: &Project| p.id == **item || p.slug == **item)
                })
                .cloned()
                .collect();

            if !not_found_items.is_empty() {
                log::trace!(
                    "Unmatched project_ids or slugs found: {:?}, added to Redis queue for processing.",
                    not_found_items
                );
                self.add_project_ids_into_queue(not_found_items).await?;
            } else {
                log::trace!("All requested projects found in the database.");
            }
        }

        Ok(projects)
    }

    pub async fn get_project_all_versions(
        &self,
        project_id_or_slug: String,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        featured: Option<bool>,
    ) -> Result<Vec<Version>, ServiceError> {
        if project_id_or_slug.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("project_id or slug"),
                reason: String::from("project_id_or_slug cannot be empty"),
            });
        }

        // 优化: 首先尝试从缓存获取映射关系
        let project_id = if project_id_or_slug.len() == 8
            && project_id_or_slug
                .chars()
                .all(|c| c.is_ascii_alphanumeric())
        {
            // 看起来像是 project_id，尝试获取对应的 slug
            if let Some(slug) = self.get_cached_slug(&project_id_or_slug).await {
                log::trace!(
                    "Found cached slug {} for project_id {}",
                    slug,
                    project_id_or_slug
                );
            }
            project_id_or_slug.clone()
        } else {
            // 看起来像是 slug，尝试获取对应的 project_id
            if let Some(cached_id) = self.get_cached_project_id(&project_id_or_slug).await {
                log::trace!(
                    "Found cached project_id {} for slug {}",
                    cached_id,
                    project_id_or_slug
                );
                cached_id
            } else {
                // 缓存未命中，需要查询数据库
                let project = self
                    .get_project_by_id_or_slug(project_id_or_slug.clone())
                    .await?;
                match project {
                    Some(p) => p.id,
                    None => {
                        return Err(ServiceError::NotFound {
                            resource: String::from("Modrinth Project"),
                            detail: Some(format!(
                                "Project with ID or slug {} not found",
                                project_id_or_slug
                            )),
                        });
                    }
                }
            }
        };

        let version_collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::Version>("modrinth_versions");

        let mut filter = doc! { "project_id": &project_id };

        if let Some(featured) = featured {
            filter.insert("featured", featured);
        }

        if let Some(game_versions) = game_versions {
            filter.insert(
                "game_versions",
                doc! { "$elemMatch": { "$in": game_versions } },
            );
        }

        if let Some(loaders) = loaders {
            filter.insert("loaders", doc! { "$elemMatch": { "$in": loaders } });
        }

        let mut cursor = version_collection.find(filter).await?;

        let mut versions = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
        {
            versions.push(doc.into());
        }

        if versions.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth Version"),
                detail: Some(format!(
                    "No versions found for project ID {}",
                    &project_id_or_slug
                )),
            });
        }

        Ok(versions)
    }

    pub async fn get_version(&self, version_id: String) -> Result<Option<Version>, ServiceError> {
        if version_id.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("version_id"),
                reason: String::from("version_id cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::Version>("modrinth_versions");

        match collection.find_one(doc! { "_id": &version_id }).await? {
            Some(doc) => Ok(Some(doc.into())),
            None => {
                self.add_version_ids_into_queue(vec![version_id.clone()])
                    .await?;
                Err(ServiceError::NotFound {
                    resource: String::from("Modrinth Version"),
                    detail: Some(format!("Version with ID {} not found", version_id)),
                })
            }
        }
    }

    pub async fn get_versions(
        &self,
        version_ids: Vec<String>,
    ) -> Result<Vec<Version>, ServiceError> {
        if version_ids.is_empty() {
            // return Err(ServiceError::InvalidInput {
            //     field: String::from("version_ids"),
            //     reason: String::from("version_ids cannot be empty"),
            // });
            return Ok(Vec::new()); // 官方返回的是 []
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::Version>("modrinth_versions");

        let filter = doc! { "_id": { "$in": &version_ids } };

        let mut cursor = collection.find(filter).await?;

        let mut versions: Vec<Version> = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
        {
            versions.push(doc.into());
        }

        if versions.is_empty() {
            self.add_version_ids_into_queue(version_ids.clone()).await?;
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth Version"),
                detail: Some(format!(
                    "No versions found for the provided version_ids: {:?}",
                    version_ids
                )),
            });
        } else {
            let not_found_version_ids: Vec<String> = version_ids
                .iter()
                .filter(|v_id| !versions.iter().any(|v| v.id == **v_id))
                .cloned()
                .collect();

            if !not_found_version_ids.is_empty() {
                self.add_version_ids_into_queue(not_found_version_ids)
                    .await?;
            } else {
                log::trace!("All requested versions found in the database.");
            }
        }

        Ok(versions)
    }

    pub async fn get_version_file(
        &self,
        hash: String,
        algorithm: String,
    ) -> Result<Option<Version>, ServiceError> {
        if hash.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("hash"),
                reason: String::from("hash cannot be empty"),
            }); // 单个 hash 请求直接返回 404
        }

        // 优化: 首先尝试从缓存获取 version_id
        if let Some(cached_version_id) = self.get_cached_version_id(&algorithm, &hash).await {
            log::trace!(
                "Found cached version_id {} for {}:{}",
                cached_version_id,
                algorithm,
                hash
            );
            return self.get_version(cached_version_id).await;
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::File>("modrinth_files");

        let filter = doc! { format!("_id.{}", algorithm): &hash };

        match collection.find_one(filter).await? {
            Some(doc) => {
                // 缓存 hash -> version_id 映射
                if let Err(e) = self
                    .cache_hash_version_mapping(&algorithm, &hash, &doc.version_id)
                    .await
                {
                    log::warn!("Failed to cache hash-version mapping: {}", e);
                }

                let version_data = self.get_version(doc.version_id).await;
                match version_data {
                    Ok(Some(version)) => Ok(Some(version)),
                    Ok(None) => Err(ServiceError::NotFound {
                        resource: String::from("Modrinth Version"),
                        detail: Some(format!(
                            "Version for file with {} {} not found",
                            algorithm, hash
                        )),
                    }),
                    Err(e) => Err(e),
                }
            }
            None => {
                self.add_hashes_into_queue(algorithm.clone(), vec![hash.clone()])
                    .await?;
                Err(ServiceError::NotFound {
                    resource: String::from("Modrinth files"),
                    detail: Some(format!("File with {} {} not found", algorithm, hash)),
                })
            }
        }
    }

    pub async fn get_version_files(
        &self,
        hashes: Vec<String>,
        algorithm: String, // "sha1" or "sha512"
    ) -> Result<MutilFilesResponse, ServiceError> {
        if hashes.is_empty() {
            // return Err(ServiceError::InvalidInput {
            //     field: String::from("hashes"),
            //     reason: String::from("hashes cannot be empty"),
            // });
            return Ok(MutilFilesResponse { entries: None }); // 官方返回的是 {}
        }

        // 查找文件
        let files_collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::File>("modrinth_files");

        let hash_field = format!("_id.{}", &algorithm);
        let files_filter = doc! { &hash_field: { "$in": &hashes } };

        let mut files_cursor = files_collection.find(files_filter).await?;
        let mut files = Vec::new();

        while let Some(doc) =
            files_cursor
                .try_next()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: format!("Failed to fetch file documents: {}", e),
                    source: Some(e),
                })?
        {
            files.push(doc);
        }

        if files.is_empty() {
            self.add_hashes_into_queue(algorithm.clone(), hashes.clone())
                .await?;
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth files"),
                detail: Some(format!(
                    "No files found for the provided {} hashes: {:?}",
                    algorithm, hashes
                )),
            });
        }

        // 提取版本 ID
        let version_ids: Vec<String> = files.iter().map(|file| file.version_id.clone()).collect();

        // // 查找版本
        // let versions_collection = self
        //     .db
        //     .database(get_database_name().as_str())
        //     .collection::<Version>("modrinth_versions");

        // let versions_filter = doc! { "_id": { "$in": &version_ids } };
        // let mut versions_cursor = versions_collection.find(versions_filter, None).await?;
        // let mut versions = Vec::new();

        // while let Some(doc) =
        //     versions_cursor
        //         .try_next()
        //         .await
        //         .map_err(|e| ServiceError::DatabaseError {
        //             message: format!("Failed to fetch version documents: {}", e),
        //             source: Some(e),
        //         })?
        // {
        //     versions.push(doc);
        // }

        let versions: Vec<Version> = self.get_versions(version_ids.clone()).await?;

        if versions.is_empty() {
            // self.get_versions 将会处理未找到的 version_ids 的队列提交
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth versions"),
                detail: Some(format!(
                    "No versions found for the provided version_ids: {:?}",
                    version_ids
                )),
            });
        }

        // 创建哈希值到版本的映射
        let mut result: HashMap<String, Version> = HashMap::new();

        for version in versions {
            if let Some(first_file) = version.files.first() {
                let hash_value = match algorithm.as_str() {
                    "sha1" => &first_file.hashes.sha1,
                    "sha512" => &first_file.hashes.sha512,
                    _ => continue,
                };
                result.insert(hash_value.clone(), version);
            }
        }

        if result.is_empty() {
            self.add_hashes_into_queue(algorithm.clone(), hashes.clone())
                .await?;
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth version files"),
                detail: Some(format!(
                    "No matching version files found for hashes: {:?}",
                    hashes
                )),
            });
        }

        // 将未找到的 hashes 添加到队列
        let not_found_hashes: Vec<String> = hashes
            .iter()
            .filter(|h| !result.contains_key(*h))
            .cloned()
            .collect();

        if !not_found_hashes.is_empty() {
            self.add_hashes_into_queue(algorithm.clone(), not_found_hashes)
                .await?;
        } else {
            log::trace!("All requested hashes found in the database.");
        }

        Ok(MutilFilesResponse {
            entries: Some(result),
        })
    }

    pub async fn get_version_file_update(
        &self,
        hash: String,
        algorithm: String,
        loaders: Vec<String>,
        game_versions: Vec<String>,
    ) -> Result<Option<Version>, ServiceError> {
        if hash.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("hash"),
                reason: String::from("hash cannot be empty"),
            }); // 单个 hash 请求直接返回 404
        }

        // 使用聚合查询从 modrinth_files 集合开始
        let files_collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("modrinth_files");

        // 构建聚合管道
        let hash_field = format!("_id.{}", algorithm);
        let mut pipeline = vec![
            doc! { "$match": { &hash_field: &hash } },
            doc! { "$project": {
                format!("_id.{}", algorithm): 1,
                "project_id": 1
            }},
            doc! { "$lookup": {
                "from": "modrinth_versions",
                "localField": "project_id",
                "foreignField": "project_id",
                "as": "versions_fields"
            }},
            doc! { "$unwind": "$versions_fields" },
        ];

        // 添加版本过滤条件
        let mut version_match = doc! {};
        if !game_versions.is_empty() {
            version_match.insert(
                "versions_fields.game_versions",
                doc! { "$in": game_versions },
            );
        }
        if !loaders.is_empty() {
            version_match.insert("versions_fields.loaders", doc! { "$in": loaders });
        }

        if !version_match.is_empty() {
            pipeline.push(doc! { "$match": version_match });
        }

        pipeline.extend([
            doc! { "$sort": { "versions_fields.date_published": -1 } },
            doc! { "$replaceRoot": { "newRoot": "$versions_fields" } },
            doc! { "$limit": 1 },
        ]);

        let mut cursor = files_collection.aggregate(pipeline).await?;

        if let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version document: {}", e),
                source: Some(e),
            })?
        {
            match bson::from_document::<db::Version>(doc) {
                Ok(version) => Ok(Some(version.into())),
                Err(e) => Err(ServiceError::UnexpectedError(format!(
                    "Failed to deserialize Version: {}",
                    e
                ))),
            }
        } else {
            self.add_hashes_into_queue(algorithm.clone(), vec![hash.clone()])
                .await?;
            Err(ServiceError::NotFound {
                resource: String::from("Modrinth version file"),
                detail: Some(format!("No matching version file found for hash {}", hash)),
            })
        }
    }

    pub async fn get_version_files_update(
        &self,
        hashes: Vec<String>,
        algorithm: String,
        loaders: Vec<String>,
        game_versions: Vec<String>,
    ) -> Result<MutilFilesResponse, ServiceError> {
        if hashes.is_empty() {
            // return Err(ServiceError::InvalidInput {
            //     field: String::from("hashes"),
            //     reason: String::from("hashes cannot be empty"),
            // });
            return Ok(MutilFilesResponse { entries: None }); // 官方返回的是 {}
        }

        // 使用聚合查询从 modrinth_files 集合开始
        let files_collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("modrinth_files");

        // 构建聚合管道
        let hash_field = format!("_id.{}", algorithm);

        let mut pipeline = vec![
            doc! { "$match": { &hash_field: { "$in": &hashes } } },
            doc! { "$project": {
                format!("_id.{}", algorithm): 1,
                "project_id": 1
            }},
            doc! { "$lookup": {
                "from": "modrinth_versions",
                "localField": "project_id",
                "foreignField": "project_id",
                "as": "versions_fields"
            }},
            doc! { "$unwind": "$versions_fields" },
        ];

        // 添加版本过滤条件
        let mut version_match = doc! {};
        if !game_versions.is_empty() {
            version_match.insert(
                "versions_fields.game_versions",
                doc! { "$in": game_versions },
            );
        }

        if !loaders.is_empty() {
            version_match.insert("versions_fields.loaders", doc! { "$in": loaders });
        }

        if !version_match.is_empty() {
            pipeline.push(doc! { "$match": version_match });
        }

        pipeline.extend([
            doc! { "$sort": { "versions_fields.date_published": -1 } },
            doc! { "$group": {
                "_id": format!("$_id.{}", algorithm),
                "latest_date": { "$first": "$versions_fields.date_published" },
                "detail": { "$first": "$versions_fields" }
            }},
        ]);

        let mut cursor = files_collection.aggregate(pipeline).await?;
        let mut result: HashMap<String, Version> = HashMap::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
        {
            if let (Some(bson::Bson::String(hash_value)), Some(bson::Bson::Document(detail_doc))) =
                (doc.get("_id"), doc.get("detail"))
            {
                match bson::from_document::<db::Version>(detail_doc.clone()) {
                    Ok(version) => {
                        result.insert(hash_value.clone(), version.into());
                    }
                    Err(e) => {
                        return Err(ServiceError::UnexpectedError(format!(
                            "Failed to deserialize Version: {}",
                            e
                        )));
                    }
                }
            }
        }

        if result.is_empty() {
            // return Err(ServiceError::NotFound {
            //     resource: String::from("Modrinth version files"),
            //     detail: Some(format!(
            //         "No matching version files found for hashes: {:?}",
            //         hashes
            //     )),
            // });
            self.add_hashes_into_queue(algorithm.clone(), hashes.clone())
                .await?;
            return Ok(MutilFilesResponse { entries: None }); // 官方返回的是 {}
        } else {
            let not_found_hashes = hashes
                .iter()
                .filter(|h| !result.contains_key(*h))
                .cloned()
                .collect::<Vec<String>>();

            if !not_found_hashes.is_empty() {
                self.add_hashes_into_queue(algorithm.clone(), not_found_hashes)
                    .await?;
            } else {
                log::trace!("All requested hashes found in the database.");
            }
        }

        Ok(MutilFilesResponse {
            entries: Some(result),
        })
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::Category>("modrinth_categories");

        let cursor = collection
            .find(doc! {})
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: e.to_string(),
                source: Some(e),
            })?;

        let db_categories: Vec<db::Category> =
            cursor
                .try_collect()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        let categories: Vec<Category> = db_categories.into_iter().map(Into::into).collect();

        Ok(categories)
    }

    pub async fn get_loaders(&self) -> Result<Vec<Loader>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::Loader>("modrinth_loaders");

        let cursor = collection
            .find(doc! {})
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: e.to_string(),
                source: Some(e),
            })?;

        let db_loaders: Vec<db::Loader> =
            cursor
                .try_collect()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        let loaders: Vec<Loader> = db_loaders.into_iter().map(Into::into).collect();

        Ok(loaders)
    }

    pub async fn get_game_versions(&self) -> Result<Vec<GameVersion>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<db::GameVersion>("modrinth_game_versions");

        let cursor = collection
            .find(doc! {})
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: e.to_string(),
                source: Some(e),
            })?;

        let db_game_versions: Vec<db::GameVersion> =
            cursor
                .try_collect()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        let game_versions: Vec<GameVersion> =
            db_game_versions.into_iter().map(Into::into).collect();

        Ok(game_versions)
    }
}

pub fn default_algorithm_from_hashes(hashes: &[String]) -> String {
    // Gets first hash, optionally
    let empty_string = "".into();
    let hash = hashes.first().unwrap_or(&empty_string);
    let hash_len = hash.len();
    // Sha1 = 40 characters
    // Sha512 = 128 characters
    // Favour sha1 as default, unless the hash is longer or equal to 128 characters
    if hash_len >= 128 {
        return "sha512".into();
    }
    "sha1".into()
}
