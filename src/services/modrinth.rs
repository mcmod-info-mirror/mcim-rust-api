use bson::doc;
use futures::stream::TryStreamExt;
use mongodb::{bson::Document, Client as Mongo_Client};
use reqwest::Client;

use crate::config::database::get_database_name;
use crate::errors::ServiceError;
use crate::models::modrinth::*;

pub struct ModrinthService {
    db: Mongo_Client,
}

impl ModrinthService {
    pub fn new(db: Mongo_Client) -> Self {
        Self { db }
    }

    pub async fn search(
        &self,
        query: Option<String>,
        facets: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
        index: Option<String>,
        modrinth_api_url: &str,
    // ) -> Result<SearchResponse, ServiceError> {
    ) -> Result<serde_json::Value, ServiceError> {
        let client = Client::new();
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
            });

        // 如果出错直接返回错误
        let response = match response {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        // // 检查响应状态码
        // if !response.status().is_success() {
        //     return Err(ServiceError::ExternalServiceError {
        //         service: String::from("Modrinth API"),
        //         message: format!("Request failed with status: {}", response.status()),
        //     });
        // }

        // let search_result: SearchResponse = response
        let search_result = response
            .json()
            .await
            .map_err(|e| ServiceError::UnexpectedError(format!("Failed to parse JSON: {}", e)))?;

        return Ok(search_result);
    }

    pub async fn get_project_by_id_or_slug(
        &self,
        project_id_or_slug: String,
    ) -> Result<Option<Project>, ServiceError> {
        if project_id_or_slug.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("project_id or slug"),
                reason: String::from("Project_id or slugcannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Project>("modrinth_projects");

        match collection
            .find_one(
                doc! { "$or": [
                    { "_id": &project_id_or_slug },
                    { "slug": &project_id_or_slug }
                ] },
                None,
            )
            .await?
        {
            Some(doc) => {
                // let project_data: Project = bson::from_document(doc).map_err(|e| {
                //     ServiceError::UnexpectedError(format!("Failed to deserialize Project: {}", e))
                // })?;

                // Ok(Some(project_data))
                Ok(Some(doc))
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("Modrinth Project"),
                detail: Some(format!(
                    "Project with project_id or slug {} not found",
                    project_id_or_slug
                )),
            }),
        }
    }

    pub async fn get_projects(
        &self,
        project_ids_or_slugs: Vec<String>,
    ) -> Result<Vec<Project>, ServiceError> {
        if project_ids_or_slugs.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("project_ids or slugs"),
                reason: String::from("Project_ids or slugs cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Project>("modrinth_projects");

        let filter = doc! {
            "$or": [
                { "_id": { "$in": project_ids_or_slugs.clone() } },
                { "slug": { "$in": project_ids_or_slugs } }
            ]
        };

        let mut cursor =
            collection
                .find(filter, None)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: format!("Failed to fetch project documents: {}", e),
                    source: Some(e),
                })?;

        let mut projects = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch project documents: {}", e),
                source: Some(e),
            })?
        {
            // match bson::from_document::<Project>(doc) {
            //     Ok(project) => projects.push(project),
            //     Err(e) => {
            //         return Err(ServiceError::UnexpectedError(format!(
            //             "Failed to deserialize Project: {}",
            //             e
            //         )));
            //     }
            // }
            projects.push(doc);
        }

        if projects.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth Project"),
                detail: Some("No projects found for the provided IDs or slugs".to_string()),
            });
        }

        Ok(projects)
    }

    pub async fn get_project_all_versions(
        &self,
        project_id: String,
    ) -> Result<Vec<Version>, ServiceError> {
        if project_id.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("project_id"),
                reason: String::from("project_id cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Version>("modrinth_versions");

        let filter = doc! { "project_id": &project_id };

        let mut cursor = collection.find(filter, None).await?;

        let mut versions = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
        {
            // match bson::from_document::<Version>(doc) {
            //     Ok(version) => versions.push(version),
            //     Err(e) => {
            //         return Err(ServiceError::UnexpectedError(format!(
            //             "Failed to deserialize Version: {}",
            //             e
            //         )));
            //     }
            // }
            versions.push(doc);
        }

        if versions.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth Version"),
                detail: Some(format!("No versions found for project ID {}", &project_id)),
            });
        }

        Ok(versions)
    }

    pub async fn get_projects_by_ids(
        &self,
        project_ids: Vec<String>,
    ) -> Result<Vec<Project>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Project>("modrinth_projects");

        let filter = doc! { "_id": { "$in": project_ids } };

        let mut cursor = collection.find(filter, None).await?;

        let mut projects = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch project documents: {}", e),
                source: Some(e),
            })?
        {
            // match bson::from_document::<Project>(doc) {
            //     Ok(project) => projects.push(project),
            //     Err(e) => {
            //         return Err(ServiceError::UnexpectedError(format!(
            //             "Failed to deserialize Project: {}",
            //             e
            //         )));
            //     }
            // }
            projects.push(doc);

            if projects.is_empty() {
                return Err(ServiceError::NotFound {
                    resource: String::from("Modrinth Project"),
                    detail: Some("No projects found for the provided IDs".to_string()),
                });
            }
        }
        Ok(projects)
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
            .collection::<Version>("modrinth_versions");

        match collection
            .find_one(doc! { "_id": &version_id }, None)
            .await?
        {
            Some(doc) => {
                // let version_data: Version = bson::from_document(doc).map_err(|e| {
                //     ServiceError::UnexpectedError(format!("Failed to deserialize Version: {}", e))
                // })?;

                // Ok(Some(version_data))
                Ok(Some(doc))
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("Modrinth Version"),
                detail: Some(format!("Version with ID {} not found", version_id)),
            }),
        }
    }

    pub async fn get_versions(
        &self,
        version_ids: Vec<String>,
    ) -> Result<Vec<Version>, ServiceError> {
        if version_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("version_ids"),
                reason: String::from("version_ids cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Version>("modrinth_versions");

        let filter = doc! { "_id": { "$in": version_ids } };

        let mut cursor = collection.find(filter, None).await?;

        let mut versions = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
        {
            // match bson::from_document::<Version>(doc) {
            //     Ok(version) => versions.push(version),
            //     Err(e) => {
            //         return Err(ServiceError::UnexpectedError(format!(
            //             "Failed to deserialize Version: {}",
            //             e
            //         )));
            //     }
            // }
            versions.push(doc);
        }

        if versions.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth Version"),
                detail: Some("No versions found for the provided IDs".to_string()),
            });
        }

        Ok(versions)
    }

    pub async fn get_version_file(
        &self,
        hash: String,
        algorithm: String,
    ) -> Result<Option<File>, ServiceError> {
        if hash.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("hash"),
                reason: String::from("hash cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<File>("modrinth_files");

        let filter = doc! { format!("_id.{}", algorithm): &hash };

        match collection.find_one(filter, None).await? {
            Some(doc) => {
                // let file_data: File = bson::from_document(doc).map_err(|e| {
                //     ServiceError::UnexpectedError(format!("Failed to deserialize File: {}", e))
                // })?;
                Ok(Some(doc))
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("Modrinth files"),
                detail: Some(format!(
                    "File with {} {} not found",
                    algorithm, hash
                )),
            }),
        }
    }

    pub async fn get_version_files(
        &self,
        hashes: Vec<String>,
        algorithm: String, // "sha1" or "sha512"
    ) -> Result<MutilFilesResponse, ServiceError> {
        if hashes.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("hashes"),
                reason: String::from("hashes cannot be empty"),
            });
        }

        // 查找文件
        let files_collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<File>("modrinth_files");

        let hash_field = format!("_id.{}", &algorithm);
        let files_filter = doc! { &hash_field: { "$in": &hashes } };

        let mut files_cursor = files_collection.find(files_filter, None).await?;
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
            // match bson::from_document::<File>(doc) {
            //     Ok(file) => files.push(file),
            //     Err(e) => {
            //         return Err(ServiceError::UnexpectedError(format!(
            //             "Failed to deserialize File: {}",
            //             e
            //         )));
            //     }
            // }
            files.push(doc);
        }

        if files.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth files"),
                detail: Some("No files found for the provided hashes".to_string()),
            });
        }

        // 提取版本 ID
        let version_ids: Vec<String> = files.iter().map(|file| file.version_id.clone()).collect();

        // 查找版本
        let versions_collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Version>("modrinth_versions");

        let versions_filter = doc! { "_id": { "$in": &version_ids } };
        let mut versions_cursor = versions_collection.find(versions_filter, None).await?;
        let mut versions = Vec::new();

        while let Some(doc) =
            versions_cursor
                .try_next()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: format!("Failed to fetch version documents: {}", e),
                    source: Some(e),
                })?
        {
            // match bson::from_document::<Version>(doc) {
            //     Ok(version) => versions.push(version),
            //     Err(e) => {
            //         return Err(ServiceError::UnexpectedError(format!(
            //             "Failed to deserialize Version: {}",
            //             e
            //         )));
            //     }
            // }
            versions.push(doc);
        }

        if versions.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth versions"),
                detail: Some("No versions found for the file version IDs".to_string()),
            });
        }

        // 创建哈希值到版本的映射
        let mut result = std::collections::HashMap::new();

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
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth version files"),
                detail: Some("No matching version files found".to_string()),
            });
        }

        Ok(MutilFilesResponse { entries: result })
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
            });
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

        let mut cursor = files_collection.aggregate(pipeline, None).await?;

        if let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version document: {}", e),
                source: Some(e),
            })?
        {
            match bson::from_document::<Version>(doc) {
                Ok(version) => Ok(Some(version)),
                Err(e) => Err(ServiceError::UnexpectedError(format!(
                    "Failed to deserialize Version: {}",
                    e
                ))),
            }
        } else {
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth version file"),
                detail: Some(format!("No matching version file found for hash {}", hash)),
            });
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
            return Err(ServiceError::InvalidInput {
                field: String::from("hashes"),
                reason: String::from("hashes cannot be empty"),
            });
        }


        // 使用聚合查询从 modrinth_files 集合开始
        let files_collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("modrinth_files");

        // 构建聚合管道
        let hash_field = format!("_id.{}", algorithm);

        let mut pipeline = vec![
            doc! { "$match": { &hash_field: { "$in": hashes } } },
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

        let mut cursor = files_collection.aggregate(pipeline, None).await?;
        let mut result = std::collections::HashMap::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!("Failed to fetch version documents: {}", e),
                source: Some(e),
            })?
        {
            if let (Some(bson::Bson::String(hash_value)), Some(bson::Bson::Document(detail_doc))) = 
            (doc.get("_id"), doc.get("detail")) {
            
            match bson::from_document::<Version>(detail_doc.clone()) {
                Ok(version) => {
                    result.insert(hash_value.clone(), version);
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
            return Err(ServiceError::NotFound {
                resource: String::from("Modrinth version files"),
                detail: Some("No matching version files found".to_string()),
            });
        }

        Ok(MutilFilesResponse { entries: result })
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Category>("modrinth_categories");

        let cursor =
            collection
                .find(doc! {}, None)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        let categories: Vec<Category> =
            cursor
                .try_collect::<Vec<Category>>()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        Ok(categories)
    }

    pub async fn get_loaders(&self) -> Result<Vec<Loader>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Loader>("modrinth_loaders");

        let cursor =
            collection
                .find(doc! {}, None)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        let loaders: Vec<Loader> =
            cursor
                .try_collect()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        Ok(loaders)
    }

    pub async fn get_game_versions(&self) -> Result<Vec<GameVersion>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<GameVersion>("modrinth_game_versions");

        let cursor =
            collection
                .find(doc! {}, None)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

        let game_versions: Vec<GameVersion> =
            cursor
                .try_collect()
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                })?;

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
