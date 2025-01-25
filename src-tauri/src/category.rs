use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tauri::api::path::config_dir;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color: String,
    pub is_productive: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CategoryConfig {
    pub categories: Vec<Category>,
    pub app_categories: HashMap<String, String>, // app_name -> category_id
}

impl CategoryConfig {
    fn create_default_categories() -> Vec<Category> {
        vec![
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Work".to_string(),
                color: "#4F46E5".to_string(), // Indigo
                is_productive: true,
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Development".to_string(),
                color: "#2563EB".to_string(), // Blue
                is_productive: true,
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Communication".to_string(),
                color: "#7C3AED".to_string(), // Purple
                is_productive: true,
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Entertainment".to_string(),
                color: "#DC2626".to_string(), // Red
                is_productive: false,
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Social Media".to_string(),
                color: "#EA580C".to_string(), // Orange
                is_productive: false,
            },
        ]
    }

    pub fn default() -> Self {
        CategoryConfig {
            categories: Self::create_default_categories(),
            app_categories: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::get_config_path()?;
        
        if !config_file.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(config_file)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_file = Self::get_config_path()?;
        
        // Garante que o diretório existe
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_file, content)?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let mut path = config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?;
        path.push("chronos-track");
        path.push("categories.json");
        Ok(path)
    }

    pub fn get_category_for_app(&self, app_name: &str) -> Option<&Category> {
        self.app_categories
            .get(app_name)
            .and_then(|category_id| {
                self.categories
                    .iter()
                    .find(|cat| &cat.id == category_id)
            })
    }

    pub fn set_app_category(&mut self, app_name: String, category_id: String) -> Result<()> {
        // Verifica se a categoria existe
        if !self.categories.iter().any(|cat| cat.id == category_id) {
            return Err(anyhow::anyhow!("Category not found: {}", category_id));
        }
        
        // Se chegou aqui, a categoria existe
        self.app_categories.insert(app_name, category_id);
        self.save()?;
        
        Ok(())
    }

    pub fn add_category(&mut self, name: String, color: String, is_productive: bool) -> Result<Category> {
        let id = uuid::Uuid::new_v4().to_string();
        let category = Category {
            id: id.clone(),
            name,
            color,
            is_productive,
        };
        self.categories.push(category.clone());
        self.save()?;
        Ok(category)
    }

    pub fn update_category(&mut self, id: String, name: String, color: String, is_productive: bool) -> Result<()> {
        if let Some(category) = self.categories.iter_mut().find(|c| c.id == id) {
            category.name = name;
            category.color = color;
            category.is_productive = is_productive;
            self.save()?;
        }
        Ok(())
    }

    pub fn delete_category(&mut self, id: &str) -> Result<()> {
        self.categories.retain(|c| c.id != id);
        self.app_categories.retain(|_, cat_id| cat_id != id);
        self.save()?;
        Ok(())
    }
} 