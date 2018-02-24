extern crate gtk;
extern crate gio;
extern crate glib;
extern crate pango;

use gtk::prelude::*;
use gio::prelude::*;
use gtk::MenuItemExt;
use gtk::MenuExt;
use glib::prelude::*;
use std::env::args;
use std::path::Path;
use std::io::BufRead;

type Result = std::result::Result<(), gtk::Error>;

//@formatter:off
macro_rules! clone {
    (@param _) => (_);
    (@param $x: ident) => ($x);
    ($($n: ident),+ => move || $body:expr) => (
        {
            $(let $n = $n.clone();)+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $(let $n = $n.clone();)+
            move |$(clone!(@param $p),)+| $body
        }
    )
}
//@formatter:on

fn main() {
    let mut app = gtk::Application::new("com.wenjun.gtkdemo", gio::ApplicationFlags::FLAGS_NONE).unwrap();
    let path = Path::new("gtkdemo.gresource");
    assert!(path.exists());
    let resources = gio::Resource::load(&path)
        .expect("resource is missing");
    gio::resources_register(&resources);

    setup_app_actions(&mut app).unwrap();
    setup_app_signals(&mut app).unwrap();

    app.run(&args().collect::<Vec<_>>());
}

fn setup_app_signals(app: &mut gtk::Application) -> Result {
    app.connect_startup(|app| {
        let builder = gtk::Builder::new_from_resource("/ui/appmenu.ui");
        let appmenu: gio::MenuModel = builder.get_object("appmenu").unwrap();

        app.set_app_menu(&appmenu);
    });

    app.connect_activate(on_app_activate);
    Ok(())
}


fn on_app_activate(app: &gtk::Application) {
    let builder = gtk::Builder::new_from_resource("/ui/main.ui");
    let window: gtk::ApplicationWindow = builder.get_object("window").unwrap();
    app.add_window(&window);

    let run_action = gio::SimpleAction::new("run", None);
    window.add_action(&run_action);
    run_action.connect_activate(on_window_run);

    let notebook: gtk::Notebook = builder.get_object("notebook").unwrap();
    let info_view: gtk::TextView = builder.get_object("info-textview").unwrap();
    let source_view: gtk::TextView = builder.get_object("source-textview").unwrap();
    let headerbar: gtk::HeaderBar = builder.get_object("headerbar").unwrap();
    let treeview: gtk::TreeView = builder.get_object("treeview").unwrap();
    let model = treeview.get_model().unwrap();
    let sw: gtk::ScrolledWindow = builder.get_object("source-scrolledwindow").unwrap();
    let sw_vscrollbar = sw.get_vscrollbar().unwrap();
    let sw_vadjustment = sw.get_vadjustment().unwrap();

    let menu = gtk::Menu::new();
    let item = gtk::MenuItem::new_with_label("Start");
    item.connect_activate_item(clone!(sw_vadjustment => move |item|{
        sw_vadjustment.set_value(sw_vadjustment.get_lower());
    }));
    menu.append(&item);

    let item = gtk::MenuItem::new_with_label("End");
    item.connect_activate_item(clone!(sw_vadjustment => move |item|{
        sw_vadjustment.set_value(sw_vadjustment.get_upper()-sw_vadjustment.get_page_size());
    }));

    sw_vscrollbar.connect_popup_menu(clone!(menu => move |_|{
        menu.popup_at_pointer(None);
        true
    }));

    window.show_all();
}

fn on_window_run(action: &gio::SimpleAction, var: &Option<glib::Variant>) {}

fn setup_app_actions(app: &mut gtk::Application) -> Result {
    let about_action = gio::SimpleAction::new("about", None);
    let quit_action = gio::SimpleAction::new("quit", None);

    app.add_action(&about_action);
    app.add_action(&quit_action);

    Ok(())
}


struct ParsedBuffer{
    info_buffer:gtk::TextBuffer,
    source_buffer:gtk::TextBuffer,
}

fn load_file(demoname: &str, filename: &str) -> std::result::Result<ParsedBuffer, gtk::Error> {
//    remove_data_tabs();
//    add_data_tab(demoname);
    let info_buffer = gtk::TextBuffer::new(None);
    let title_tag = gtk::TextTag::new("title");
    title_tag.set_property_font(Some("Sans 18"));
    title_tag.set_property_pixels_below_lines(10);

    let resource_filename = "/sources/".to_owned() + filename;
    let bytes = gio::resources_lookup_data(&resource_filename, gio::ResourceLookupFlags::empty())?;

    let (title, info,source) = parse_title_and_description(&bytes);

    // info buffer
    let info_buffer = gtk::TextBuffer::new(None);
    let mut start = info_buffer.get_start_iter();
    let mut end = info_buffer.get_start_iter();
    info_buffer.insert(&mut end, &title);
    info_buffer.apply_tag_by_name("title", &start, &end);
    info_buffer.insert(&mut end, &info);

    // source buffer
    let source_buffer = gtk::TextBuffer::new(None);
    let mut start = source_buffer.get_start_iter();
    source_buffer.insert(&mut start,&source);

    Ok(ParsedBuffer{
        info_buffer,
        source_buffer,
    })
}

fn fontify(source_buffer: &gtk::TextBuffer){
    let source_tag = gtk::TextTag::new("source");
    source_tag.set_property_font(Some("monospace"));

    let comment_tag = gtk::TextTag::new("comment");
    comment_tag.set_property_foreground(Some("DodgerBlue"));

    let type_tag = gtk::TextTag::new("type");
    type_tag.set_property_foreground(Some("ForestGreen"));

    let string_tag = gtk::TextTag::new("string");
    string_tag.set_property_foreground(Some("RosyBrown"));
    string_tag.set_property_weight()
}

fn parse_title_and_description(bytes: &glib::Bytes) -> (String, String, String) {
    let mut lines = bytes.lines().filter_map(|it| it.ok());
    let mut title: Vec<String> = lines.by_ref()
        .take_while(|l|{
            let trimmed = l.trim();
            !(trimmed.is_empty())
        })
        .collect();

    for l in title.iter_mut() {
        if let Some(idx) = l.find('*') {
            l.drain(..idx + 1);
            if l.starts_with('/') {
                l.remove(0);
            }
        }
        println!("{}",l);
    }
    let info = title.split_off(1).join("").trim().to_string();
    let title = title[0].split('/').collect::<Vec<_>>()[1].to_string();
    let source = lines.collect::<Vec<String>>().join("");

    println!("{},{},{}",title,info,source);
    (title, info, source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title_and_description() {
        let b = glib::Bytes::from_static(b"/* Theming/Multiple Backgrounds\n\
 *\n\
 * Gtk themes are written using CSS.\n\
 *\n\
 */\n\
 \n\
 fn main(){}\n");
        let (title, info, source) = parse_title_and_description(&b);
        println!("{:?}", title);
        assert_eq!("Multiple Backgrounds", title);

        println!("{}", info);
        assert_eq!("Gtk themes are written using CSS.", info);

        println!("{}",source);
        assert_eq!("fn main(){}",source);
    }
}
