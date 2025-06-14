use bson::doc;
use futures::stream::TryStreamExt;
use mongodb::{bson::Document, Client};

use crate::config::database::get_database_name;
use crate::models::curseforge::entities::{Category, File, Fingerprint, Mod};
use crate::models::curseforge::responses::*;

// use crate::services::ServiceError;
use crate::errors::ServiceError;

pub struct CurseforgeService {
    db: Client,
}

impl CurseforgeService {
    pub fn new(db: Client) -> Self {
        Self { db }
    }

    pub async fn get_mod(&self, mod_id: i32) -> Result<Option<ModResponse>, ServiceError> {
        if mod_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_id"),
                reason: String::from("Mod ID cannot be negative"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("curseforge_mods");

        match collection.find_one(doc! { "_id": mod_id }, None).await? {
            Some(doc) => {
                let mod_data: Mod = bson::from_document(doc).map_err(|e| {
                    ServiceError::UnexpectedError(format!("Failed to deserialize Mod: {}", e))
                })?;

                let response = ModResponse { data: mod_data };
                Ok(Some(response))
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("Mod"),
                detail: Some(format!("Mod with ID {} not found", mod_id)),
            }),
        }
    }

    pub async fn get_mods(&self, mod_ids: Vec<i32>) -> Result<ModsResponse, ServiceError> {
        if mod_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_ids"),
                reason: String::from("Mod IDs cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("curseforge_mods");

        let mut cursor = collection
            .find(doc! { "_id": { "$in": mod_ids } }, None)
            .await?;

        let mut mods = Vec::new();

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| ServiceError::Database {
                message: "Failed to fetch mods from database".to_string(),
                source: Some(e),
            })?
        {
            match bson::from_document::<Mod>(doc) {
                Ok(mod_data) => mods.push(mod_data),
                Err(e) => {
                    return Err(ServiceError::UnexpectedError(format!(
                        "Failed to deserialize Mod: {}",
                        e
                    )));
                }
            }
        }
        // empty 则直接返回 { "data": [] }
        if mods.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Mods"),
                detail: Some("No mods found for the provided IDs".to_string()),
            });
        }
        Ok(ModsResponse { data: mods })
    }

    pub async fn get_mod_by_slug(&self, slug: &str) -> Result<Option<ModResponse>, ServiceError> {
        if slug.trim().is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("slug"),
                reason: String::from("Slug cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("curseforge_mods");

        match collection
            .find_one(doc! { "slug": slug }, None)
            .await
            .map_err(|e| ServiceError::Database {
                message: "Failed to fetch mod by slug".to_string(),
                source: Some(e),
            })? {
            Some(doc) => {
                let mod_data: Mod = bson::from_document(doc).map_err(|e| {
                    ServiceError::UnexpectedError(format!("Failed to deserialize Mod: {}", e))
                })?;

                let response = ModResponse { data: mod_data };
                Ok(Some(response))
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("Mod"),
                detail: Some(format!("Mod with slug '{}' not found", slug)),
            }),
        }
    }

    pub async fn get_file(&self, file_id: i32) -> Result<FileResponse, ServiceError> {
        if file_id.is_negative() {
            return Err(ServiceError::InvalidInput {
                field: String::from("file_id"),
                reason: String::from("File ID cannot be negative"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("curseforge_files");

        match collection
            .find_one(doc! { "_id": file_id }, None)
            .await
            .map_err(|e| ServiceError::Database {
                message: "Failed to fetch file by ID".to_string(),
                source: Some(e),
            })? {
            Some(doc) => {
                let file_data: File = bson::from_document(doc).map_err(|e| {
                    ServiceError::UnexpectedError(format!("Failed to deserialize File: {}", e))
                })?;

                let response = FileResponse { data: file_data };
                Ok(response)
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("File"),
                detail: Some(format!("File with ID {} not found", file_id)),
            }),
        }
    }

    pub async fn get_files(&self, file_ids: Vec<i32>) -> Result<FilesResponse, ServiceError> {
        if file_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("file_ids"),
                reason: String::from("File IDs cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("curseforge_files");

        let mut cursor = collection
            .find(doc! { "_id": { "$in": file_ids } }, None)
            .await?;

        let mut files = Vec::new();
        while let Ok(Some(doc)) = cursor.try_next().await.map_err(|e| ServiceError::Database {
            message: String::from("Failed to fetch files from database"),
            source: Some(e),
        }) {
            let file_data: File = bson::from_document(doc).map_err(|e| {
                ServiceError::UnexpectedError(format!("Failed to deserialize File: {}", e))
            })?;
            files.push(file_data);
        }

        if files.is_empty() {
            return Err(ServiceError::NotFound {
                resource: String::from("Files"),
                detail: Some("No files found for the provided IDs".to_string()),
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
                reason: String::from("Mod ID cannot be negative"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("curseforge_files");

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
                        doc! { "$skip": index },
                        doc! { "$limit": page_size }
                    ],
                    "count": [
                        doc! { "$count": "total" }
                    ]
                }
            },
        ];

        let mut cursor =
            collection
                .aggregate(pipeline, None)
                .await
                .map_err(|e| ServiceError::Database {
                    message: String::from("Failed to aggregate mod files"),
                    source: Some(e),
                })?;

        let result = cursor.try_next().await.map_err(|e| ServiceError::Database {
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
                reason: String::from("Mod ID and File ID cannot be negative"),
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
        fingerprints: Vec<i32>,
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
            .collection::<Document>("curseforge_fingerprints");

        // 可选 game_id 参数用于过滤
        let mut filter = doc! { "_id": { "$in": &fingerprints } };
        if let Some(game_id) = game_id {
            filter.insert("gameId", game_id);
        }

        let mut cursor = collection.find(filter, None).await?;
        let mut fingerprint_results = Vec::new();

        while let Ok(Some(doc)) = cursor.try_next().await.map_err(|e| ServiceError::Database {
            message: String::from("Failed to fetch fingerprints from database"),
            source: Some(e),
        }) {
            if let Ok(fingerprint) = bson::from_document::<Fingerprint>(doc) {
                fingerprint_results.push(fingerprint);
            } else {
                return Err(ServiceError::UnexpectedError(String::from(
                    "Failed to deserialize Fingerprint",
                )));
            }
        }

        let exact_fingerprints = fingerprint_results.iter().map(|f| f.id).collect();

        let unmatched_fingerprints = fingerprints
            .into_iter()
            .filter(|f| !fingerprint_results.iter().any(|fp| fp.id == *f))
            .collect();

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
    ) -> Result<Vec<Category>, ServiceError> {
        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<Document>("curseforge_categories");

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
                .map_err(|e| ServiceError::Database {
                    message: String::from("Failed to fetch categories from database"),
                    source: Some(e),
                })?;

        let mut categories = Vec::new();
        while let Ok(Some(doc)) = cursor.try_next().await.map_err(|e| ServiceError::Database {
            message: String::from("Failed to fetch categories from database"),
            source: Some(e),
        }) {
            let category: Category = bson::from_document(doc).map_err(|e| {
                ServiceError::UnexpectedError(format!("Failed to deserialize Category: {}", e))
            })?;
            categories.push(category);
        }

        Ok(categories)
    }
}
