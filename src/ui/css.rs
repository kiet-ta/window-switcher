use gtk4::CssProvider;

pub fn load_css() {
    let provider = CssProvider::new();
    let css_data = r#"
        window {
            background-color: rgba(20, 20, 25, 0.7);
        }
        
        .main-container {
            background: rgba(30, 30, 40, 0.95);
            border-radius: 24px; /* Bo góc tinh tế */
            padding: 24px;
            box-shadow: 0px 10px 30px rgba(0, 0, 0, 0.5);
        }

        .title-label {
            font-size: 24pt;
            font-weight: bold;
            color: white;
        }

    .window-frame {
        background-color: #2A2A35;
        border-radius: 16px;
        padding: 10px;
        /* Hiệu ứng viền gradient khi hover/focus */
        border: 2px solid transparent;
        background-clip: padding-box;
        transition: all 0.15s cubic-bezier(0.25, 1, 0.5, 1);
    }

    .window-frame:focus-within {
        background-image: linear-gradient(#2A2A35, #2A2A35), linear-gradient(135deg, #00ffcc, #7000ff);
        background-origin: border-box;
        background-clip: content-box, border-box;
        transform: scale(1.05); /* UX Playbook: Phản hồi thị giác rõ ràng */
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
