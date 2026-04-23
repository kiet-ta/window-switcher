use gtk4::CssProvider;

pub fn load_css() {
    let provider = CssProvider::new();
    let css_data = r#"
        window {
            background-color: rgba(10, 14, 22, 0.58);
        }

        .overlay-root {
            padding: 8px;
        }

        .main-container {
            background: linear-gradient(180deg, rgba(18, 24, 34, 0.96), rgba(11, 15, 22, 0.92));
            border-radius: 24px;
            padding: 24px 28px;
            border: 1px solid rgba(255, 255, 255, 0.08);
            box-shadow: 0 24px 80px rgba(0, 0, 0, 0.42);
        }

        .title-label {
            color: #f7fbff;
            font-size: 24pt;
            font-weight: 700;
        }

        .status-label {
            color: rgba(226, 236, 247, 0.82);
            font-size: 11pt;
            font-weight: 500;
        }

        .grid-scroll {
            background: transparent;
        }

        .window-grid {
            padding: 4px;
        }

        .window-frame {
            background: rgba(17, 22, 31, 0.92);
            border-radius: 20px;
            padding: 14px;
            border: 1px solid rgba(255, 255, 255, 0.06);
            transition: all 120ms ease-out;
        }

        .window-frame.selected-window {
            border-color: rgba(79, 195, 247, 0.92);
            box-shadow: 0 0 0 2px rgba(79, 195, 247, 0.18);
        }

        .window-frame.active-window {
            background: rgba(24, 33, 47, 0.96);
            border-color: rgba(255, 196, 107, 0.68);
        }

        .window-frame.thumbnail-pending {
            border-style: dashed;
        }

        .window-frame.thumbnail-failed {
            border-color: rgba(255, 110, 110, 0.68);
        }

        .window-frame.thumbnail-missing {
            border-color: rgba(255, 255, 255, 0.1);
        }

        .thumbnail-stack,
        .thumbnail-fallback {
            background: linear-gradient(180deg, rgba(31, 39, 54, 0.95), rgba(20, 25, 34, 0.94));
            border-radius: 16px;
        }

        .window-title {
            color: #f7fbff;
            font-size: 13pt;
            font-weight: 650;
        }

        .window-meta {
            color: rgba(205, 217, 229, 0.82);
            font-size: 10.5pt;
        }

        .window-state {
            color: rgba(118, 200, 255, 0.9);
            font-size: 10pt;
            font-weight: 600;
        }
    "#;

    provider.load_from_data(css_data);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
