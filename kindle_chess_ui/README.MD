### Frontend API crate structure
kindle-chess-ui/
├── src/
│   ├── lib.rs           // Public API
│   ├── app.rs           // Main application state machine
│   ├── renderer.rs      // X11 rendering abstraction
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── board.rs     // Chess board widget
│   │   ├── sidebar.rs   // Game info sidebar
│   │   ├── menu.rs      // Menu system
│   │   └── button.rs    // Reusable button widget
│   ├── events.rs        // Event definitions
│   └── layout.rs        // Layout management
