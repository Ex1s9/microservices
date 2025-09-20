# GameHub - Статус проекта и архитектура

## 🎯 Общая архитектура

GameHub - микросервисная платформа для управления играми, построенная на Rust с использованием gRPC и HTTP API.

### Архитектурная схема:
```
Frontend (React/Vue) 
    ↓ HTTP REST API
Gateway Service (Actix-web) :8080
    ↓ gRPC
User Service :50051  ←→  Game Service :50052
    ↓                        ↓
PostgreSQL Database :5432
```

## 📁 Структура проекта

```
gamehub/
├── services/
│   ├── user-service/          # Управление пользователями
│   ├── game-service/          # Управление играми  
│   ├── gateway-service/       # HTTP API Gateway
│   └── product-service/       # Продукты (заглушка)
├── proto/                     # Protocol Buffers схемы
│   ├── user.proto
│   └── game.proto
├── common/                    # Общие утилиты
└── docker-compose.yml         # PostgreSQL контейнер
```

## ✅ Что готово

### 1. User Service (Порт 50051) - ГОТОВ ✅
**Статус:** Полностью реализован и работает

**Функциональность:**
- ✅ gRPC сервер с полным CRUD для пользователей
- ✅ База данных с таблицей users
- ✅ Валидация данных
- ✅ Хеширование паролей (bcrypt)
- ✅ Роли пользователей: Player, Developer, Admin
- ✅ Миграции базы данных

**API методы:**
- `GetUser(id)` - получение пользователя
- `CreateUser(email, username, password, role)` - создание
- `UpdateUser(id, поля)` - обновление
- `DeleteUser(id)` - удаление
- `ListUsers(limit, offset, role)` - список с пагинацией

### 2. Game Service (Порт 50052) - ГОТОВ ✅  
**Статус:** Полностью реализован

**Функциональность:**
- ✅ gRPC сервер для управления играми
- ✅ HTTP API сервер (Axum) на порту 8080
- ✅ База данных с таблицей games
- ✅ Сложная модель игры с категориями, тегами, скриншотами
- ✅ Статусы игр: Draft, Under Review, Published, Suspended
- ✅ Поиск и фильтрация игр
- ✅ Миграции базы данных

**API методы:**
- `CreateGame()` - создание игры
- `GetGame(id)` - получение игры
- `UpdateGame(id, поля)` - обновление
- `DeleteGame(id, developer_id)` - удаление
- `ListGames(фильтры, пагинация, сортировка)` - поиск игр

### 3. Gateway Service (Порт 8080) - ГОТОВ ✅
**Статус:** Полностью реализован для User API

**Функциональность:**
- ✅ HTTP REST API gateway на Actix-web
- ✅ Middleware: rate limiting, request ID, CORS
- ✅ Логирование запросов
- ✅ Проксирование к User Service через gRPC
- ✅ Обработка ошибок и статус кодов

**HTTP Endpoints:**
- `POST /api/users` - создание пользователя
- `GET /api/users/{id}` - получение пользователя  
- `PUT /api/users/{id}` - обновление пользователя
- `DELETE /api/users/{id}` - удаление пользователя
- `GET /api/users?limit&offset` - список пользователей

### 4. База данных - НАСТРОЕНА ✅
**Статус:** Работает

**Компоненты:**
- ✅ PostgreSQL в Docker контейнере
- ✅ Роль postgres создана
- ✅ База данных gamehub создана
- ✅ Таблица users с индексами
- ✅ Таблица games с индексами и триггерами
- ✅ Миграции применены

### 5. Protocol Buffers - ГОТОВЫ ✅
**Статус:** Полностью определены

**Схемы:**
- ✅ user.proto - полное API для пользователей
- ✅ game.proto - полное API для игр
- ✅ Enum'ы для ролей, категорий, статусов

## ⚠️ Что нужно доработать

### 1. Gateway Service - Game API (Частично)
**Статус:** Нужно добавить проксирование к Game Service

**Что нужно:**
- 🔄 HTTP endpoints для игр (/api/games/*)
- 🔄 Подключение к Game Service gRPC клиенту
- 🔄 Конвертация HTTP ↔ gRPC для игр

### 2. Product Service - НЕ РЕАЛИЗОВАН ❌
**Статус:** Только заглушка (Hello World)

**Что нужно:**
- 🔄 Определить назначение сервиса
- 🔄 Реализовать функциональность
- 🔄 Добавить в gateway

### 3. Безопасность и аутентификация - НЕ РЕАЛИЗОВАНА ❌
**Что нужно:**
- 🔄 JWT токены
- 🔄 Middleware для авторизации
- 🔄 Защита endpoints по ролям
- 🔄 Login/logout API

### 4. Связи между сервисами - ЧАСТИЧНО ❌
**Что нужно:**
- 🔄 Связь игр с разработчиками (developer_id → users.id)
- 🔄 Валидация существования пользователей при создании игр
- 🔄 Каскадные операции

## 🚀 Как запустить

### Автоматический запуск:
```bash
# Запуск всех сервисов
./run-services.sh
```

### Ручной запуск:
```bash
# 1. База данных
docker-compose up -d

# 2. User Service  
cargo run -p user-service

# 3. Game Service
cargo run -p game-service  

# 4. Gateway Service
cargo run -p gateway-service
```

### Порты:
- **PostgreSQL:** 5432
- **User Service gRPC:** 50051
- **Game Service gRPC:** 50052
- **Game Service HTTP:** 8080 (дублирует функции)
- **Gateway HTTP API:** 8080

## 🧪 Тестирование

```bash
# Тест создания пользователя
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","username":"testuser","password":"password123","role":"player"}'

# Тест получения пользователя  
curl http://localhost:8080/api/users/{user_id}
```

## 📝 Следующие шаги

1. **Приоритет 1:** Добавить Game API в Gateway Service
2. **Приоритет 2:** Реализовать аутентификацию и авторизацию
3. **Приоритет 3:** Добавить связи между сервисами
4. **Приоритет 4:** Определить и реализовать Product Service
5. **Приоритет 5:** Добавить интеграционные тесты

## 💾 База данных

**Строка подключения:** `postgresql://postgres:123456789@localhost:5432/gamehub`

**Таблицы:**
- `users` (id, email, username, password_hash, role, created_at, updated_at)
- `games` (id, name, description, developer_id, categories[], tags[], price, status, и др.)

---
*Обновлено: 2025-09-20*