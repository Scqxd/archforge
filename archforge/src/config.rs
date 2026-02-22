//! Управление конфигурацией для ArchForge

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Конфигурация для ArchForge
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub ai: AiConfig,
    pub build: BuildConfig,
    pub aur: AurConfig,
    pub tui: TuiConfig,
}

/// Общие настройки
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Подробный вывод
    pub verbose: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Директория кэша
    pub cache_dir: Option<String>,
}

/// Настройки AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// AI провайдер
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// API ключ (не хранится в конфиге из соображений безопасности)
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Модель для использования
    pub model: Option<String>,
    /// Таймаут запросов в секундах
    pub timeout_secs: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: String::from("chutes"),
            api_key: None,
            model: None,
            timeout_secs: 30,
        }
    }
}

/// Настройки сборки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Флаги для makepkg
    pub makepkg_flags: Vec<String>,
    /// Параллельные задачи
    pub parallel_jobs: usize,
    /// Очистка после сборки
    pub clean_after_build: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            makepkg_flags: vec![
                String::from("--noconfirm"),
                String::from("--needed"),
            ],
            parallel_jobs: 4,
            clean_after_build: false,
        }
    }
}

/// Настройки AUR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurConfig {
    /// URL RPC для AUR
    pub rpc_url: String,
    /// URL для клонирования репозиториев
    pub clone_url: String,
}

impl Default for AurConfig {
    fn default() -> Self {
        Self {
            rpc_url: String::from("https://aur.archlinux.org/rpc"),
            clone_url: String::from("https://aur.archlinux.org"),
        }
    }
}

/// Настройки TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Тема оформления
    pub theme: String,
    /// Показывать анимации
    pub show_animations: bool,
    /// Режим Vim (hjkl навигация)
    pub vim_mode: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: String::from("default"),
            show_animations: true,
            vim_mode: true,
        }
    }
}

impl Config {
    /// Получить директорию конфигурации
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("archforge")
    }

    /// Получить путь к файлу конфигурации
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Загрузить конфигурацию из файла или создать по умолчанию
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            // Создать конфигурацию по умолчанию
            let default_config = Self::default();
            default_config.save()?;
            eprintln!("Создана конфигурация по умолчанию: {}", config_path.display());
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)?;

        // Парсинг TOML
        let config: Self = toml::from_str(&content)
            .map_err(|e| format!("Ошибка парсинга конфигурации: {}", e))?;

        Ok(config)
    }

    /// Сохранить конфигурацию в файл
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();

        // Создать директорию конфигурации если не существует
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Ошибка сериализации конфигурации: {}", e))?;

        fs::write(&config_path, content)?;

        Ok(())
    }

    /// Получить директорию кэша
    pub fn cache_dir(&self) -> PathBuf {
        self.general.cache_dir
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| dirs::cache_dir().map(|d| d.join("archforge")))
            .unwrap_or_else(|| PathBuf::from(".cache/archforge"))
    }

    /// Получить API ключ AI из переменной окружения
    ///
    /// Безопасность: API ключ НЕ хранится в файле конфигурации для предотвращения
    /// случайной утечки. Установите переменную окружения CHUTES_API_KEY.
    pub fn api_key(&self) -> Option<String> {
        std::env::var("CHUTES_API_KEY").ok()
    }
}
