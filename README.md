# EchoMesh Windows Client (`echomesh-windows`)

Нативный легковесный Windows-клиент для защищенного меш-мессенджера EchoMesh в стиле Windows 11 Fluent Design (Mica-эффект).

---

## Архитектура и стек технологий

- **Фреймворк:** Tauri v2 (быстрый нативный бэкенд на Rust + WebView2).
- **Бэкенд:** Rust с прямым подключением ядра `echomesh-core` как локального крэйта (`path = "../echomesh-core"`), разделяя общий `tokio`-рантайм без прослоек маршалинга UniFFI.
- **Интерфейс:** React 18 + TypeScript + Microsoft Fluent UI v9 (`@fluentui/react-components`, `@fluentui/react-icons`).
- **Сетевой уровень:** Noise Protocol (NK pattern), маскировка под TLS 1.3 ClientHello (Pseudo-TLS) с авторизационным токеном в `ClientHello.random`, фиксированный размер кадров 1420 байт (Wire MTU).
- **Хранилище:** Встроенный SQLite (`rusqlite`) в системной директории `%APPDATA%/EchoMesh/echomesh.db`.

---

## Структура проекта

```text
echomesh-windows/
├── package.json                 # Зависимости React, Fluent UI v9, Tauri API
├── tsconfig.json                # Конфигурация TypeScript
├── vite.config.ts               # Конфигурация сборщика Vite под Tauri v2
├── index.html                   # Входной HTML-документ
├── src/                         # Фронтенд (Fluent UI / React)
│   ├── main.tsx                 # Точка входа React, FluentProvider (Windows 11 Dark Theme)
│   ├── App.tsx                  # Главный контейнер, подписки на IPC события, автоподключение
│   ├── types.ts                 # Типы данных (MessageDto, ContactDto, RelayStatusDto)
│   ├── styles.css               # Стили Windows 11 Mica, акриловые эффекты, скроллбары
│   └── components/
│       ├── Sidebar.tsx          # Статус ноды (зеленый/красный), принудительное подключение, Echo Node, контакты
│       ├── ChatView.tsx         # Шапка шифрования (ChaCha20-Poly1305 • 1420b), бабблы, RTT, чипы
│       └── AddContactModal.tsx  # Модалка добавления пира с валидацией 64-символьного Hex-ключа
└── src-tauri/                   # Нативный бэкенд Tauri v2
    ├── Cargo.toml               # Зависимости: echomesh-core, tauri v2, tokio, serde, notification
    ├── tauri.conf.json          # Конфигурация окна (Mica), трей, NSIS-инсталлятор
    ├── build.rs                 # Сборщик Tauri
    └── src/
        ├── lib.rs               # Инициализация приложения, системный трей, сворачивание в трей
        ├── main.rs              # Точка входа бинарника
        ├── state.rs             # Глобальное состояние AppState, мост событий CoreEventsListener
        └── commands/            # Tauri IPC команды
            ├── mod.rs
            ├── connection.rs    # connect_to_relay, disconnect, get_status
            ├── contacts.rs      # add_contact, get_contacts
            └── chat.rs          # send_message, get_history
```

---

## Возможности и системная интеграция Windows

1. **Интеграция с Windows Shell:**
   - **Системный трей (System Tray Icon):** Меню с пунктами «Открыть EchoMesh», «Статус соединения», «Выход». Двойной или одинарный клик по иконке восстанавливает окно.
   - **Сворачивание в трей:** Нажатие на кнопку закрытия окна («X») сворачивает клиент в трей без остановки фонового сетевого сокета `tokio::spawn`.
   - **Всплывающие уведомления (Toast Notifications):** При получении входящих сообщений, когда окно свернуто, вызывается нативное уведомление Windows.
   - **Путь к данным:** База данных и конфиги сохраняются строго в `%APPDATA%/EchoMesh/`.

2. **Пользовательский интерфейс Fluent UI:**
   - Индикатор статуса релея (`77.81.5.109:8443`): зеленый (Online + пинг в мс), желтый (Connecting), красный (Offline).
   - Кнопка «Принудительное подключение» для быстрого переподключения к релею.
   - Фиксированный диалог **Echo Relay Node** (`[0xEE; 32]`) для loopback-тестирования.
   - Быстрые чипы тестирования: `[Ping]`, `[Noise Payload]`, `[1420b Test]` (максимальный размер кадра 1394 байта полезной нагрузки).
   - Отображение статусов сообщений (Отправляется, Отправлено, Эхо получено с RTT в мс).
   - Валидация 64-символьного Hex-ключа при добавлении контакта.

---

## Сборка и запуск

### 1. Установка зависимостей фронтенда
```bash
npm install
```

### 2. Запуск в режиме разработки (Dev)
```bash
npm run tauri dev
```

### 3. Релизная сборка для Windows (MSVC)
Сборка автономного инсталлятора (.exe через NSIS / .msi):
```bash
cargo tauri build --target x86_64-pc-windows-msvc
```
Готовый установщик появится в `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/`.
