use bson::doc;
use futures::stream::TryStreamExt;
use mongodb::Client as Mongo_Client;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use reqwest::Client;
use std::sync::Arc;

use crate::config::database::get_database_name;
use crate::errors::ServiceError;
use crate::models::curseforge::entities::{Category, File, Fingerprint, Mod};
use crate::models::curseforge::requests::SearchQuery;
use crate::models::curseforge::responses::*;

pub struct CurseforgeService {
    db: Mongo_Client,
    redis: Arc<MultiplexedConnection>,
}

impl CurseforgeService {
    pub fn new(db: Mongo_Client, redis: Arc<MultiplexedConnection>) -> Self {
        Self { db, redis }
    }

    async fn add_modids_into_queue(&self, mod_ids: Vec<i32>) -> Result<(), ServiceError> {
        if mod_ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.redis.as_ref().clone();
        let _ = conn
            .sadd::<&str, &Vec<i32>, ()>("curseforge_modids", &mod_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add modIds to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added modIds to Redis queue: {:?}", mod_ids);
        Ok(())
    }

    async fn add_fileids_into_queue(&self, file_ids: Vec<i32>) -> Result<(), ServiceError> {
        if file_ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.redis.as_ref().clone();
        let _ = conn
            .sadd::<&str, &Vec<i32>, ()>("curseforge_fileids", &file_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add fileIds to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added fileIds to Redis queue: {:?}", file_ids);
        Ok(())
    }

    async fn add_fingerprints_into_queue(
        &self,
        fingerprints: Vec<i64>,
    ) -> Result<(), ServiceError> {
        if fingerprints.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();
        let _ = conn
            .sadd::<&str, &Vec<i64>, ()>("curseforge_fingerprints", &fingerprints)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add fingerprints to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added fingerprints to Redis queue: {:?}", fingerprints);
        Ok(())
    }

    async fn check_search_result(&self, data: &serde_json::Value) -> Result<(), ServiceError> {
        if data.is_null() || !data.is_object() {
            return Err(ServiceError::UnexpectedError(
                "Search result is null or not an object".to_string(),
            ));
        }

        let mods = data.get("data").and_then(|d| d.as_array()).ok_or_else(|| {
            ServiceError::UnexpectedError("Search result does not contain 'data' array".to_string())
        })?;

        let mut mod_ids = Vec::new();

        for _mod in mods {
            if let Some(mod_id) = _mod.get("id").and_then(|id| id.as_i64()) {
                if mod_id >= 30000 {
                    mod_ids.push(mod_id as i32);
                }
            }
        }

        if mod_ids.is_empty() {
            log::debug!("Search result is empty or no valid modIds found.");
            return Ok(());
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Mod>("curseforge_mods");

        let mut cursor = collection
            .find(doc! { "_id": { "$in": &mod_ids } }, None)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: "Failed to fetch mods from database".to_string(),
                source: Some(e),
            })?;

        let mut found_mod_ids = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: "Failed to fetch mods from database".to_string(),
                source: Some(e),
            })?
        {
            found_mod_ids.push(doc.id);
        }

        let not_found_mod_ids: Vec<i32> = mod_ids
            .iter()
            .filter(|id| !found_mod_ids.contains(id))
            .cloned()
            .collect();

        if !not_found_mod_ids.is_empty() {
            self.add_modids_into_queue(not_found_mod_ids).await?;
        } else {
            log::debug!("All Mods have been found in the database.");
        }

        Ok(())
    }

    pub async fn search_mods(
        &self,
        query: &SearchQuery,
        curseforge_api_url: &str,
        curseforge_api_key: &str,
    ) -> Result<serde_json::Value, ServiceError> {
        let mut params: Vec<(&str, String)> = Vec::new();

        macro_rules! add_param {
            ($field:ident, $key:expr) => {
                if let Some(value) = &query.$field {
                    params.push(($key, value.to_string()));
                }
            };
            ($field:ident, $key:expr, transform) => {
                if let Some(value) = query.$field {
                    params.push(($key, value.to_string()));
                }
            };
        }

        let game_id = query.game_id.unwrap_or(432);
        let index = query.index.unwrap_or(0);
        let page_size = query.page_size.unwrap_or(50);

        params.push(("gameId", game_id.to_string()));
        params.push(("index", index.to_string()));
        params.push(("pageSize", page_size.to_string()));

        add_param!(class_id, "classId", transform);
        add_param!(category_id, "categoryId", transform);
        add_param!(category_ids, "categoryIds");
        add_param!(game_version, "gameVersion");
        add_param!(game_versions, "gameVersions");
        add_param!(search_filter, "searchFilter");
        add_param!(sort_field, "sortField");
        add_param!(sort_order, "sortOrder");
        add_param!(mod_loader_type, "modLoaderType");
        add_param!(mod_loader_types, "modLoaderTypes");
        add_param!(game_version_type_id, "gameVersionTypeId", transform);
        add_param!(author_id, "authorId", transform);
        add_param!(primary_author_id, "primaryAuthorId", transform);
        add_param!(slug, "slug");

        let response = Client::new()
            .get(format!("{}/v1/mods/search", curseforge_api_url))
            .header("x-api-key", curseforge_api_key)
            .query(&params)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError {
                service: "Curseforge API".into(),
                message: format!("Failed to send request: {}", e),
            })?;

        // 检查状态码
        if !response.status().is_success() {
            return Err(ServiceError::ExternalServiceError {
                service: "Curseforge API".into(),
                message: format!("Request failed with status code: {}", response.status()),
            });
        }

        let search_result = response.json::<serde_json::Value>().await.map_err(|e| {
            ServiceError::ExternalServiceError {
                service: "Curseforge API".into(),
                message: format!("Failed to parse JSON: {}", e),
            }
        })?;

        // 检查搜索结果，不存在则添加到队列
        self.check_search_result(&search_result).await?;

        Ok(search_result)
    }

    pub async fn get_mod(&self, mod_id: i32) -> Result<Option<ModResponse>, ServiceError> {
        if mod_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_id"),
                reason: String::from("ModId cannot be negative"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Mod>("curseforge_mods");

        match collection.find_one(doc! { "_id": mod_id }, None).await? {
            Some(mod_data) => {
                let response = ModResponse { data: mod_data };
                Ok(Some(response))
            }
            None => {
                // 不存在则添加到队列
                self.add_modids_into_queue(vec![mod_id]).await?;

                Err(ServiceError::NotFound {
                    resource: String::from("Mod"),
                    detail: Some(format!("Mod with modId {} not found", mod_id)),
                })
            }
        }
    }

    pub async fn get_mods(&self, mod_ids: Vec<i32>) -> Result<ModsResponse, ServiceError> {
        if mod_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_ids"),
                reason: String::from("ModIds cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Mod>("curseforge_mods");

        let mut cursor = collection
            .find(doc! { "_id": { "$in": &mod_ids } }, None)
            .await?;

        let mut mods = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: "Failed to fetch mods from database".to_string(),
                source: Some(e),
            })?
        {
            mods.push(doc);
        }

        // empty 则直接返回 { "data": [] }
        if mods.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Mods"),
                detail: Some(format!(
                    "No mods found for the provided modIds: {:?}",
                    mod_ids
                )),
            });
        }

        // 检查是否有未找到的 mod_id
        let found_mod_ids: Vec<i32> = mods.iter().map(|m| m.id).collect();
        let not_found_mod_ids: Vec<i32> = mod_ids
            .into_iter()
            .filter(|id| !found_mod_ids.contains(id))
            .collect();
        if !not_found_mod_ids.is_empty() {
            log::debug!(
                "modIds not found in database: {:?}, adding to queue for processing.",
                not_found_mod_ids
            );
            self.add_modids_into_queue(not_found_mod_ids).await?;
        } else {
            log::debug!("All Mods have been found in the database.");
        }

        Ok(ModsResponse { data: mods })
    }

    pub async fn get_file(&self, file_id: i32) -> Result<FileResponse, ServiceError> {
        if file_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("file_id"),
                reason: String::from("FileId cannot be negative"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<File>("curseforge_files");

        match collection
            .find_one(doc! { "_id": file_id }, None)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: "Failed to fetch file by ID".to_string(),
                source: Some(e),
            })? {
            Some(file_data) => {
                let response = FileResponse { data: file_data };
                Ok(response)
            }
            None => {
                self.add_fileids_into_queue(vec![file_id]).await?;
                Err(ServiceError::NotFound {
                    resource: String::from("File"),
                    detail: Some(format!("File with fileId {} not found", file_id)),
                })
            }
        }
    }

    pub async fn get_files(&self, file_ids: Vec<i32>) -> Result<FilesResponse, ServiceError> {
        if file_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("file_ids"),
                reason: String::from("FileIds cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<File>("curseforge_files");

        let mut cursor = collection
            .find(doc! { "_id": { "$in": &file_ids } }, None)
            .await?;

        let mut files = Vec::new();
        while let Ok(Some(doc)) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: String::from("Failed to fetch files from database"),
                source: Some(e),
            })
        {
            files.push(doc);
        }

        // 检查是否有未找到的 file_id
        let found_file_ids: Vec<i32> = files.iter().map(|f| f.id).collect();
        let not_found_file_ids: Vec<i32> = file_ids
            .iter()
            .filter(|id| !found_file_ids.contains(id))
            .cloned()
            .collect();
        if !not_found_file_ids.is_empty() {
            self.add_fileids_into_queue(not_found_file_ids).await?;
        } else {
            log::debug!("All Files have been found in the database.");
        }

        if files.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Files"),
                detail: Some(format!(
                    "No files found for the provided fileIds: {:?}",
                    file_ids
                )),
            });
        }

        Ok(FilesResponse { data: files })
    }

    pub async fn get_mod_files(
        &self,
        mod_id: i32,
        game_version: Option<String>,
        mod_loader_type: Option<i32>,
        index: Option<i32>,
        page_size: Option<i32>,
    ) -> Result<ModFilesResponse, ServiceError> {
        if mod_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_id"),
                reason: String::from("ModId cannot be negative"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<File>("curseforge_files");

        let mut filter = doc! { "modId": mod_id };
        let mut game_version_filters = Vec::new();

        if let Some(version) = game_version {
            game_version_filters.push(version);
        }

        if let Some(loader_type) = mod_loader_type {
            let loader_type_str = match loader_type {
                1 => "Forge",
                2 => "Cauldron",
                3 => "LiteLoader",
                4 => "Fabric",
                5 => "Quilt",
                6 => "NeoForge",
                _ => {
                    return Err(ServiceError::UnexpectedError(String::from(
                        "Invalid mod loader type",
                    )))
                }
            };
            game_version_filters.push(loader_type_str.into());
        }

        if !game_version_filters.is_empty() {
            filter.insert("gameVersions", doc! { "$all": game_version_filters });
        }

        let index = index.unwrap_or(0);
        let page_size = page_size.unwrap_or(50);

        let pipeline = vec![
            doc! { "$match": filter },
            doc! {
                "$facet": {
                    "data": [
                        doc! { "$sort": { "fileDate": -1 } },
                        doc! { "$skip": index },
                        doc! { "$limit": page_size }
                    ],
                    "count": [
                        doc! { "$count": "total" }
                    ]
                }
            },
        ];

        let mut cursor = collection.aggregate(pipeline, None).await.map_err(|e| {
            ServiceError::DatabaseError {
                message: String::from("Failed to aggregate mod files"),
                source: Some(e),
            }
        })?;

        let result = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: String::from("Failed to fetch mod files"),
                source: Some(e),
            });

        let (files, total_count) = if let Ok(Some(doc)) = result {
            // 获取数据
            let data_array: &Vec<bson::Bson> = doc.get_array("data").map_err(|_| {
                ServiceError::UnexpectedError("Failed to get data array".to_string())
            })?;

            let mut files = Vec::new();
            for item in data_array {
                if let Some(file_doc) = item.as_document() {
                    let file_data: File = bson::from_document(file_doc.clone()).map_err(|e| {
                        ServiceError::UnexpectedError(format!("Failed to deserialize File: {}", e))
                    })?;
                    files.push(file_data);
                }
            }

            // 获取总数
            let count_array = doc.get_array("count").map_err(|_| {
                ServiceError::UnexpectedError("Failed to get count array".to_string())
            })?;

            let total_count = if let Some(count_item) = count_array.first() {
                count_item
                    .as_document()
                    .and_then(|d| d.get_i32("total").ok())
                    .unwrap_or(0)
            } else {
                0
            };

            (files, total_count)
        } else {
            (Vec::new(), 0)
        };
        
        // 有筛选条件很容易为空，不能当作 Mod 不存在
        // if total_count == 0 {
        //     self.add_modids_into_queue(vec![mod_id]).await?;
        // }

        let result_count = files.len() as i32;
        Ok(ModFilesResponse {
            data: files,
            pagination: Pagination {
                index: index as i32,
                page_size: page_size as i32,
                result_count,
                total_count,
            },
        })
    }

    pub async fn get_file_download_url(
        &self,
        mod_id: i32,
        file_id: i32,
    ) -> Result<DownloadUrlResponse, ServiceError> {
        if mod_id.is_negative() || file_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_id or file_id"),
                reason: String::from("ModId and FileId cannot be negative"),
            });
        }

        let file = self.get_file(file_id).await?;

        let file_data = file;
        Ok(DownloadUrlResponse {
            data: file_data.data.download_url.unwrap_or_default(),
        })
    }

    pub async fn get_fingerprints(
        &self,
        fingerprints: Vec<i64>,
        game_id: Option<i32>,
    ) -> Result<FingerprintResponse, ServiceError> {
        if fingerprints.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("fingerprints"),
                reason: String::from("Fingerprints cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Fingerprint>("curseforge_fingerprints");

        // 可选 game_id 参数用于过滤
        let mut filter = doc! { "_id": { "$in": &fingerprints } };
        if let Some(game_id) = game_id {
            filter.insert("file.gameId", game_id);
        }

        let mut cursor = collection.find(filter, None).await?;
        let mut fingerprint_results = Vec::new();

        while let Ok(Some(doc)) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: String::from("Failed to fetch fingerprints from database"),
                source: Some(e),
            })
        {
            fingerprint_results.push(doc);
        }

        let exact_fingerprints = fingerprint_results.iter().map(|f| f.id).collect();

        let unmatched_fingerprints: Vec<i64> = fingerprints
            .into_iter()
            .filter(|f| !fingerprint_results.iter().any(|fp| fp.id == *f))
            .collect();

        if unmatched_fingerprints.is_empty() {
            log::debug!("All fingerprints have been found in the database.");
        } else {
            self.add_fingerprints_into_queue(unmatched_fingerprints.clone())
                .await?;
        }

        let response = FingerprintResponse {
            data: FingerprintResult {
                is_cache_built: true,
                exact_matches: fingerprint_results,
                exact_fingerprints: exact_fingerprints,
                installed_fingerprints: Vec::new(),
                unmatched_fingerprints: unmatched_fingerprints,
            },
        };

        Ok(response)
    }

    pub async fn get_categories(
        &self,
        game_id: i32,
        class_id: Option<i32>,
        class_only: Option<bool>,
    ) -> Result<CategoriesResponse, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Category>("curseforge_categories");

        // 构建查询过滤器
        let mut filter = doc! { "gameId": game_id };

        if class_id.is_some() {
            filter.insert("classId", class_id);
        } else if class_only.unwrap_or(false) {
            filter.insert("isClass", true);
        }

        let mut cursor =
            collection
                .find(filter, None)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: String::from("Failed to fetch categories from database"),
                    source: Some(e),
                })?;

        let mut categories = Vec::new();
        while let Ok(Some(doc)) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: String::from("Failed to fetch categories from database"),
                source: Some(e),
            })
        {
            // categories.push(CategoryResponseObject::from(doc));
            categories.push(doc);
        }

        if categories.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Categories"),
                detail: Some(format!(
                    "No categories found for gameId {} and classId {:?} and classOnly {:?}",
                    game_id,
                    class_id,
                    class_only.unwrap_or(false)
                )),
            });
        }

        Ok(CategoriesResponse { data: categories })
    }
}
