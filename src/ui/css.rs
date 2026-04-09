use gtk4::CssProvider;

pub fn load_css() {
    let provider = CssProvider::new();
    let css_data = r#"
        window {
            background-color: rgba(15, 15, 20, 0.85); 
            border-radius: 15px;
        }
        
        .main-container {
            padding: 30px;
        }

        .title-label {
            font-size: 24pt;
            font-weight: bold;
            color: white;
        }

        .window-frame {
            background-color: rgba(40, 40, 50, 0.9);
            border-radius: 10px;
            padding: 15px;
            border: 2px solid transparent;
            transition: all 0.2s ease-in-out;
        }

        .window-frame:hover, .window-frame:focus-within {
            border: 2px solid #00ffcc;
            background-color: rgba(60, 60, 80, 0.9);
            box-shadow: 0 0 10px rgba(0, 255, 204, 0.5);
        }

        .window-title {
            color: white;
            font-size: 14pt;
            font-weight: 500;
        }
    "#;
    
    provider.load_from_data(css_data);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
