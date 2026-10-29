use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use reqwest::Client;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::curseforge as repo;
use crate::errors::ServiceError;
use crate::models::curseforge::requests::SearchQuery;
use crate::models::curseforge::responses::*;

pub struct CurseforgeService {
    db: PgPool,
    redis: Arc<MultiplexedConnection>,
}

impl CurseforgeService {
    pub fn new(db: PgPool, redis: Arc<MultiplexedConnection>) -> Self {
        Self { db, redis }
    }

    // error=[POST] 400 https://api.curseforge.com/v1/mods/files {"value":"one or more ids are not valid"}
    // 用于同步的 ID 必须大于 0，否则会 400
    
    async fn add_modids_into_queue(&self, mod_ids: Vec<i32>) -> Result<(), ServiceError> {
        let filtered_mod_ids = mod_ids.iter().filter(|id| *id >= & 30000).cloned().collect::<Vec<i32>>();
        if filtered_mod_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();
        conn.sadd::<&str, &Vec<i32>, ()>("curseforge_modids", &filtered_mod_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add modIds to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added modIds to Redis queue: {:?}", filtered_mod_ids);
        Ok(())
    }

    async fn add_fileids_into_queue(&self, file_ids: Vec<i32>) -> Result<(), ServiceError> {
        let filtered_file_ids = file_ids.iter().filter(|id| *id >= & 0).cloned().collect::<Vec<i32>>();
        if filtered_file_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.as_ref().clone();
        conn.sadd::<&str, &Vec<i32>, ()>("curseforge_fileids", &filtered_file_ids)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add fileIds to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added fileIds to Redis queue: {:?}", filtered_file_ids);
        Ok(())
    }

    async fn add_fingerprints_into_queue(
        &self,
        fingerprints: Vec<i64>,
    ) -> Result<(), ServiceError> {
        let filtered_fingerprints = fingerprints
            .iter()
            .filter(|f| *f >= &0)
            .cloned()
            .collect::<Vec<i64>>();
        if filtered_fingerprints.is_empty() {
            return Ok(());
        }

        let filtered_fingerprints = fingerprints
            .iter()
            .filter(|f| *f >= &0)
            .cloned()
            .collect::<Vec<i64>>();

        let mut conn = self.redis.as_ref().clone();
        conn.sadd::<&str, &Vec<i64>, ()>("curseforge_fingerprints", &filtered_fingerprints)
            .await
            .map_err(|e| -> ServiceError {
                ServiceError::ExternalServiceError {
                    service: "Redis".into(),
                    message: format!("Failed to add fingerprints to Redis queue: {}", e),
                }
            })?;
        log::debug!("Added fingerprints to Redis queue: {:?}", filtered_fingerprints);
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

        let found_mod_ids = repo::existing_mod_ids(&self.db, &mod_ids)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: "Failed to fetch mods from database".to_string(),
                source: Some(e),
            })?;

        let not_found_mod_ids: Vec<i32> = mod_ids
            .iter()
            .filter(|id| !found_mod_ids.contains(id))
            .cloned()
            .collect();

        if !not_found_mod_ids.is_empty() {
            self.add_modids_into_queue(not_found_mod_ids).await?;
        } else {
            log::trace!("All Mods have been found in the database.");
        }

        Ok(())
    }

    pub async fn search_mods(
        &self,
        client: &Client,
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

        let response = client
            .get(format!("{}/v1/mods/search", curseforge_api_url))
            .header("x-api-key", curseforge_api_key)
            .query(&params)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError {
                service: "Curseforge API".into(),
                message: format!("Failed to send request: {}", e),
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ServiceError::ExternalServiceError {
                service: "Curseforge API".into(),
                message: format!(
                    "Request failed with status: {}, response: {}",
                    status,
                    response
                        .text()
                        .await
                        .unwrap_or_else(|_| "No response text".to_string())
                ),
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ServiceError::ExternalServiceError {
                service: "Curseforge API".into(),
                message: format!("Failed to read response body: {}", e),
            })?;
        let search_result: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| ServiceError::ExternalServiceError {
                service: "Curseforge API".into(),
                message: format!(
                    "Failed to parse JSON: {}, text: {}",
                    e,
                    String::from_utf8_lossy(&bytes)
                ),
            })?;

        // 检查有无未缓存的 Project
        match self.check_search_result(&search_result).await {
            Ok(_) => log::trace!("Curseforge check_search_result completed successfully"),
            Err(e) => log::error!("Curseforge check_search_result failed: {}", e),
        };

        Ok(search_result)
    }

    pub async fn get_mod(&self, mod_id: i32) -> Result<Option<ModResponse>, ServiceError> {
        if mod_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_id"),
                reason: String::from("ModId cannot be negative"),
            });
        }

        match repo::get_mods(&self.db, &[mod_id]).await?.pop() {
            Some(mod_data) => {
                // let response = ModResponse { data: mod_data };
                let response = ModResponse {
                    data: mod_data.into(),
                };
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

        let mods =
            repo::get_mods(&self.db, &mod_ids)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: "Failed to fetch mods from database".to_string(),
                    source: Some(e),
                })?;

        // 检查是否有未找到的 mod_id
        let found_mod_ids: Vec<i32> = mods.iter().map(|m| m.id).collect();
        let not_found_mod_ids: Vec<i32> = mod_ids
            .iter()
            .copied()
            .filter(|id| !found_mod_ids.contains(id))
            .collect();
        if !not_found_mod_ids.is_empty() {
            log::debug!(
                "modIds not found in database: {:?}, adding to queue for processing.",
                not_found_mod_ids
            );
            self.add_modids_into_queue(not_found_mod_ids).await?;
        } else {
            log::trace!("All Mods have been found in the database.");
        }

        /*
        curl -i -X 'POST' \
          'https://api.curseforge.com/v1/mods' \
          -H 'accept: application/json' \
          -H 'Content-Type: application/json' \
          -H 'x-api-key: $API_KEY' \
          -d '{
          "filterPcOnly": true,
          "modIds": [
            238222112
          ]
        }'
        HTTP/2 404
        content-length: 0
        date: Sat, 12 Sep 2026 07:06:03 GMT
        x-cache: Error from cloudfront
        via: 1.1 715cc2fe99ad4f311de46e74c166c5a2.cloudfront.net (CloudFront)
        x-amz-cf-pop: LOS50-P4
        x-amz-cf-id: EGanUHlrFZiDD8YG6hWQpedqhiWQ30pKnB7DX8CBBNyTsyixGDDjNQ==
        */
        // 不会返回 Body，可以任意填写
        // 但是必须返回 404，不能返回 200
        if mods.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Mods"),
                detail: Some(format!(
                    "No mods found for the provided modIds: {:?}",
                    mod_ids
                )),
            });
        }

        let response_mods = mods.into_iter().map(|m| m.into()).collect();
        Ok(ModsResponse {
            data: response_mods,
        })
    }

    pub async fn get_file(&self, file_id: i32) -> Result<FileResponse, ServiceError> {
        if file_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("file_id"),
                reason: String::from("FileId cannot be negative"),
            });
        }

        match repo::get_files(&self.db, &[file_id])
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: "Failed to fetch file by ID".to_string(),
                source: Some(e),
            })?
            .pop()
        {
            Some(file_data) => {
                let response = FileResponse {
                    data: file_data.into(),
                };
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

        let files = repo::get_files(&self.db, &file_ids).await.map_err(|e| {
            ServiceError::DatabaseError {
                message: String::from("Failed to fetch files from database"),
                source: Some(e),
            }
        })?;

        // 检查是否有未找到的 file_id
        let found_file_ids: Vec<i32> = files.iter().map(|f| f.info.id).collect();
        let not_found_file_ids: Vec<i32> = file_ids
            .iter()
            .filter(|id| !found_file_ids.contains(id))
            .cloned()
            .collect();
        if !not_found_file_ids.is_empty() {
            self.add_fileids_into_queue(not_found_file_ids).await?;
        } else {
            log::trace!("All Files have been found in the database.");
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

        // Ok(FilesResponse { data: files })
        let response_files = files.into_iter().map(|f| f.into()).collect();
        Ok(FilesResponse {
            data: response_files,
        })
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
                    )));
                }
            };
            game_version_filters.push(loader_type_str.into());
        }

        let index = index.unwrap_or(0);
        let page_size = page_size.unwrap_or(50);

        let (files, total_count) = repo::get_mod_files(
            &self.db,
            mod_id,
            &game_version_filters,
            i64::from(index),
            i64::from(page_size),
        )
        .await
        .map_err(|e| ServiceError::DatabaseError {
            message: String::from("Failed to fetch mod files"),
            source: Some(e),
        })?;
        let total_count = i32::try_from(total_count).unwrap_or(i32::MAX);

        // 有筛选条件很容易为空，不能当作 Mod 不存在
        // if total_count == 0 {
        //     self.add_modids_into_queue(vec![mod_id]).await?;
        // }

        let result_count = files.len() as i32;
        Ok(ModFilesResponse {
            data: files.into_iter().map(|f| f.into()).collect(),
            pagination: Pagination {
                index,
                page_size,
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

        let installed_fingerprints = fingerprints.clone();

        let file_results: Vec<File> =
            repo::get_files_by_fingerprints(&self.db, &fingerprints, game_id)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: String::from("Failed to fetch fingerprints from database"),
                    source: Some(e),
                })?
                .into_iter()
                .map(Into::into)
                .collect();

        let exact_fingerprints = file_results
            .iter()
            .filter_map(|f| f.file_fingerprint)
            .collect();

        // 查询 Mod，获取 latestFiles
        let mod_ids = file_results.iter().map(|f| f.mod_id).collect::<Vec<_>>();
        let mod_latest_files_results: HashMap<i32, Vec<FileInfo>> =
            repo::get_latest_files(&self.db, &mod_ids)
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: String::from("Failed to fetch mods from database"),
                    source: Some(e),
                })?
                .into_iter()
                .map(|(mod_id, files)| (mod_id, files.into_iter().map(Into::into).collect()))
                .collect();

        let unmatched_fingerprints: Vec<i64> = fingerprints
            .into_iter()
            .filter(|f| {
                !file_results
                    .iter()
                    .any(|file| file.file_fingerprint == Some(*f))
            })
            .collect();

        // 将 id 设置为 fingerprint.file.modId
        // https://github.com/Meloong-Git/PCL/issues/6656
        let exact_matches = file_results
            .into_iter()
            .map(|f| Fingerprint {
                id: f.mod_id,
                latest_files: mod_latest_files_results
                    .get(&f.mod_id)
                    .cloned()
                    .unwrap_or_default(),
                file: f,
            })
            .collect();

        if unmatched_fingerprints.is_empty() {
            log::trace!("All fingerprints have been found in the database.");
        } else {
            self.add_fingerprints_into_queue(unmatched_fingerprints.clone())
                .await?;
            log::debug!(
                "Unmatched fingerprints found: {:?}, added to Redis queue for processing.",
                unmatched_fingerprints
            );
        }

        let response = FingerprintResponse {
            data: FingerprintResult {
                is_cache_built: true, // 默认值，没见过 false
                exact_matches,
                exact_fingerprints,
                installed_fingerprints,
                unmatched_fingerprints: Some(unmatched_fingerprints),
                partial_matches: Vec::new(), // 暂时不处理 partialMatches，不知道干嘛的
                partial_match_fingerprints: HashMap::new(), // 暂时不处理 partialMatchFingerprints，不知道干嘛的
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
        let categories =
            repo::get_categories(&self.db, game_id, class_id, class_only.unwrap_or(false))
                .await
                .map_err(|e| ServiceError::DatabaseError {
                    message: String::from("Failed to fetch categories from database"),
                    source: Some(e),
                })?;

        if categories.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Categories"),
                detail: Some(format!(
                    "No categories found for gameId {} and classId {:?} and classesOnly {:?}",
                    game_id,
                    class_id,
                    class_only.unwrap_or(false)
                )),
            });
        }

        Ok(CategoriesResponse {
            data: categories.into_iter().map(|c| c.into()).collect(),
        })
    }
}
