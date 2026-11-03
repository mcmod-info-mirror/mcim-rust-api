use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use reqwest::Client;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::modrinth::{self as repo, HashAlgorithm};
use crate::errors::ServiceError;
use crate::models::modrinth::responses::*;

pub struct ModrinthService {
    db: PgPool,
    redis: Arc<MultiplexedConnection>,
}

impl ModrinthService {
    pub fn new(db: PgPool, redis: Arc<MultiplexedConnection>) -> Self {
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

        let filtered_project_ids = project_ids
            .into_iter()
            .filter(|id| id.len() == 8 && id.chars().all(|c| c.is_ascii_alphanumeric()))
            .collect::<Vec<String>>();

        if filtered_project_ids.is_empty() {
            log::debug!("No valid project IDs to add to Redis queue.");
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();
        conn.sadd::<&str, &Vec<String>, ()>("modrinth_project_ids", &filtered_project_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add project ids to Redis queue: {}", e),
                }
            })?;
        log::debug!(
            "Added project ids to Redis queue: {:?}",
            filtered_project_ids
        );
        Ok(())
    }

    async fn add_version_ids_into_queue(
        &self,
        version_ids: Vec<String>,
    ) -> Result<(), ServiceError> {
        if version_ids.is_empty() {
            return Ok(());
        }

        let filtered_version_ids = version_ids
            .into_iter()
            .filter(|id| id.len() == 8 && id.chars().all(|c| c.is_ascii_alphanumeric()))
            .collect::<Vec<String>>();

        if filtered_version_ids.is_empty() {
            log::debug!("No valid version IDs to add to Redis queue.");
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();
        conn.sadd::<&str, &Vec<String>, ()>("modrinth_version_ids", &filtered_version_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add version ids to Redis queue: {}", e),
                }
            })?;
        log::debug!(
            "Added version ids to Redis queue: {:?}",
            filtered_version_ids
        );
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

        let algo_lower = algorithm.to_lowercase();

        // 校验 algorithm 是否为支持的值
        if algo_lower != "sha1" && algo_lower != "sha512" {
            return Ok(());
        }

        // 确保所有 hash 都符合规范
        let filtered_hashes = hashes
            .into_iter()
            .filter(|hash| is_valid_hash(&algo_lower, hash))
            .collect::<Vec<String>>();

        if filtered_hashes.is_empty() {
            log::debug!(
                "No valid hashes to add to Redis queue for algorithm {}.",
                algo_lower
            );
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();

        conn.sadd::<&str, &Vec<String>, ()>(
            format!("modrinth_hashes_{}", algorithm).as_str(),
            &filtered_hashes,
        )
        .await
        .map_err(|e| -> ServiceError {
            ServiceError::ExternalServiceError {
                service: "Redis".into(),
                message: format!("Failed to add hash to Redis queue: {}", e),
            }
        })?;
        log::debug!(
            "Added {}:{} to Redis queue",
            algorithm,
            filtered_hashes.join(",")
        );
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

        let found_project_ids = repo::existing_project_ids(&self.db, &project_ids)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch project documents: {}", e),
                source: Some(e),
            })?;

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

        // 在此处要打断 4xx/5xx 错误，返回 ServiceError::ExternalServiceError，以免被缓存
        let response = client
            .get(api_url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError {
                service: String::from("Modrinth API"),
                message: format!("Failed to send request: {}", e),
            })?
            .error_for_status()
            .map_err(|e| ServiceError::ExternalServiceError {
                service: String::from("Modrinth API"),
                message: format!("Upstream HTTP error: {}", e),
            })?;

        // 在此处要打断 4xx/5xx 错误，返回 ServiceError::ExternalServiceError，以免被缓存
        if let Err(e) = response.error_for_status_ref() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("<failed to read body>"));
            return Err(ServiceError::ExternalServiceError {
                service: String::from("Modrinth API"),
                message: format!("Upstream HTTP error: {}, body: {}", e, error_body),
            });
        }

        let status: reqwest::StatusCode = response.status();
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

        match repo::get_project_by_id_or_slug(&self.db, &project_id_or_slug).await? {
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
        // 构建查询条件：包含已解析的project_id和剩余的待查询项
        let mut all_items_to_query = resolved_project_ids;
        all_items_to_query.extend(remaining_items.clone());

        let db_projects = repo::get_projects(&self.db, &all_items_to_query)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch project documents: {}", e),
                source: Some(e),
            })?;

        let mut projects: Vec<Project> = Vec::with_capacity(db_projects.len());
        for doc in db_projects {
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

        let versions: Vec<Version> = repo::get_project_versions(
            &self.db,
            &project_id,
            featured,
            game_versions.as_deref(),
            loaders.as_deref(),
        )
        .await
        .map_err(|e| ServiceError::DatabaseError {
            message: format!("Failed to fetch version documents: {}", e),
            source: Some(e),
        })?
        .into_iter()
        .map(Into::into)
        .collect();

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

        match repo::get_versions(&self.db, std::slice::from_ref(&version_id))
            .await?
            .pop()
        {
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

        let versions: Vec<Version> = repo::get_versions(&self.db, &version_ids)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
            .into_iter()
            .map(Into::into)
            .collect();

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

        let file = match HashAlgorithm::from_name(&algorithm) {
            Some(algo) => repo::get_files_by_hashes(&self.db, algo, std::slice::from_ref(&hash))
                .await?
                .pop(),
            None => None,
        };

        match file {
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
        let files = match HashAlgorithm::from_name(&algorithm) {
            Some(algo) => repo::get_files_by_hashes(&self.db, algo, &hashes)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: format!("Failed to fetch file documents: {}", e),
                    source: Some(e),
                })?,
            None => Vec::new(),
        };

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

        // 用 HashSet 来优化查找速度，避免 Vec contains O(n) 查找
        let requested_hashes_set: std::collections::HashSet<&str> =
            hashes.iter().map(|s| s.as_str()).collect();

        for version in versions {
            // 此处导致只有是第一个文件的哈希值会被映射到版本，其他文件的哈希值不会被映射
            // if let Some(first_file) = version.files.first() {
            //     let hash_value = match algorithm.as_str() {
            //         "sha1" => &first_file.hashes.sha1,
            //         "sha512" => &first_file.hashes.sha512,
            //         _ => continue,
            //     };
            //     result.insert(hash_value.clone(), version);
            // }

            // 遍历版本的所有文件，找到匹配的哈希值
            for file in &version.files {
                let hash_value = match algorithm.as_str() {
                    "sha1" => &file.hashes.sha1,
                    "sha512" => &file.hashes.sha512,
                    _ => continue,
                };
                // 只有客户端查询列表中包含的 hash，才插入到结果中
                if requested_hashes_set.contains(hash_value.as_str()) {
                    result.insert(hash_value.clone(), version.clone());
                }
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

        let version_id = match HashAlgorithm::from_name(&algorithm) {
            Some(algo) => repo::get_latest_version_ids_for_hashes(
                &self.db,
                algo,
                std::slice::from_ref(&hash),
                &loaders,
                &game_versions,
            )
            .await?
            .pop()
            .map(|(_, version_id)| version_id),
            None => None,
        };
        let version = match version_id {
            Some(version_id) => repo::get_versions(&self.db, &[version_id]).await?.pop(),
            None => None,
        };

        if let Some(version) = version {
            Ok(Some(version.into()))
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

        // 每个哈希 -> 同一 Project 中满足过滤条件的最新 Version
        let latest = match HashAlgorithm::from_name(&algorithm) {
            Some(algo) => {
                repo::get_latest_version_ids_for_hashes(
                    &self.db,
                    algo,
                    &hashes,
                    &loaders,
                    &game_versions,
                )
                .await?
            }
            None => Vec::new(),
        };
        let version_ids: Vec<String> = latest.iter().map(|(_, id)| id.clone()).collect();
        let versions: HashMap<String, Version> = repo::get_versions(&self.db, &version_ids)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
            .into_iter()
            .map(|v| (v.id.clone(), v.into()))
            .collect();

        let mut result: HashMap<String, Version> = HashMap::new();
        for (hash_value, version_id) in latest {
            if let Some(version) = versions.get(&version_id) {
                result.insert(hash_value, version.clone());
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
        let db_categories = repo::get_categories(&self.db).await?;

        let categories: Vec<Category> = db_categories.into_iter().map(Into::into).collect();

        Ok(categories)
    }

    pub async fn get_loaders(&self) -> Result<Vec<Loader>, ServiceError> {
        let db_loaders = repo::get_loaders(&self.db).await?;

        let loaders: Vec<Loader> = db_loaders.into_iter().map(Into::into).collect();

        Ok(loaders)
    }

    pub async fn get_game_versions(&self) -> Result<Vec<GameVersion>, ServiceError> {
        let db_game_versions = repo::get_game_versions(&self.db).await?;

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

fn is_valid_hash(algorithm: &str, hash: &str) -> bool {
    let expected_len = match algorithm.to_lowercase().as_str() {
        "sha1" => 40,
        "sha512" => 128,
        _ => return false, // 不支持的算法直接判定无效
    };

    // 检查长度且全部字符为 Hex (0-9, a-f, A-F)
    hash.len() == expected_len && hash.chars().all(|c| c.is_ascii_hexdigit())
}
