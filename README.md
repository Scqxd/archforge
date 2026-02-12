# ArchForge

AI-powered TUI for PKGBUILD generation and AUR management.

## Описание

ArchForge — это инструмент командной строки и TUI для генерации PKGBUILD файлов из описания на естественном языке и управления пакетами AUR.

## Установка

### Из AUR (рекомендуется)

```bash
# Using yay
yay -S archforge-git

# Using paru
paru -S archforge-git
```

### Из исходников

```bash
git clone https://github.com/Scqxd/archforge.git
cd archforge
cargo build --release
./target/release/archforge --version
```

## Быстрый старт

### Генерация PKGBUILD

```bash
# Описание пакета -> готовый PKGBUILD
archforge generate "Консольный hello world на C"
archforge generate "Neovim с поддержкой Python и Lua"
archforge generate "Firefox с VAAPI"
```

### Интерактивный режим

```bash
archforge interactive
# или просто
archforge
```

---

## AI Генерация PKGBUILD

ArchForge использует Chutes API (MiniMaxAI) для интеллектуальной генерации PKGBUILD.

### Настройка API ключа

**Вариант 1: Переменная окружения**
```bash
export CHUTES_API_KEY="твой_ключ"
archforge generate "пакет"
```

**Вариант 2: Флаг --api-key**
```bash
archforge generate "пакет" --api-key "твой_ключ"
```

### Выбор AI провайдера

```bash
# Chutes API (MiniMaxAI/MiniMax-M2.1-TEE) - по умолчанию
archforge generate "пакет" --ai-provider chutes

# Локальная модель (в разработке)
archforge generate "пакет" --ai-provider local

# OpenAI (в разработке)
archforge generate "пакет" --ai-provider openai
```

### Примеры

```bash
# Простой C/C++ проект
archforge generate "Hello world на C"

# Go приложение
archforge generate "HTTP сервер на Go"

# Python утилита
archforge generate "Парсер JSON на Python"

# Сложный пакет
archforge generate "Neovim с LSP для Rust и autocomplete"
```

---

## Команды

### generate — Генерация PKGBUILD

```bash
archforge generate "описание" [опции]

Опции:
  -o, --output FILE     Сохранить в файл
  -q, --quiet           Только вывод PKGBUILD
  -a, --ai-provider     Выбор AI провайдера (chutes/local/openai)
      --api-key         API ключ

Примеры:
  archforge generate "firefox" -o PKGBUILD
  archforge generate "hello" --api-key "ключ"
```

### search — Поиск в AUR

```bash
archforge search "запрос" [опции]

Опции:
  -j, --json    JSON вывод
  -l, --limit   Лимит результатов (по умолчанию 20)

Примеры:
  archforge search neovim
  archforge search firefox -j
```

### info — Информация о пакете

```bash
archforge info <имя_пакета>
```

### build — Сборка пакета

```bash
archforge build <PKGBUILD/директория> [опции]

Опции:
  -i, --install    Установить после сборки
      --nodeps     Пропустить проверку зависимостей
```

### init — Создание проекта

```bash
archforge init <имя> [опции]

Опции:
  -t, --template   Шаблон (basic)
  -d, --directory  Директория
```

### interactive — Интерактивный TUI

```bash
archforge interactive
```

### status — Статус системы

```bash
archforge status
```

### cache — Управление кэшем

```bash
archforge cache stats     # Статистика кэша
archforge cache models    # Очистить кэш моделей
archforge cache builds    # Очистить кэш сборок
archforge cache all       # Очистить всё
```

---

## Fallback шаблоны

Если AI недоступен, ArchForge автоматически использует шаблоны с автоопределением языка:

| Ключевые слова | makedepends | Особенности |
|----------------|-------------|-------------|
| C, C++ | gcc, make | `make` сборка |
| Go, golang | go | `go build` |
| Python | python, pip | `setup.py` |
| (по умолчанию) | gcc, make | Простой C шаблон |

---

## Конфигурация

Файл: `~/.config/archforge/config.toml`

```toml
[general]
verbose = false
cache_dir = "~/.cache/archforge"

[ai]
# Chutes API ключ (или используй CHUTES_API_KEY env)
provider = "chutes"

[build]
makepkg_flags = ["--noconfirm", "--needed"]
parallel_jobs = 4

[aur]
rpc_url = "https://aur.archlinux.org/rpc"
```

---

## Требования

- Rust 1.75+
- cargo
- makepkg (base-devel)
- git

## Лицензия

MIT License

## Ссылки

- GitHub: https://github.com/Scqxd/archforge
- AUR: https://aur.archlinux.org/packages/archforge-git