use penrose::{
    contrib::{
        actions::{create_or_switch_to_workspace, update_monitors_via_xrandr},
        extensions::{dmenu::*, Scratchpad},
        hooks::{DefaultWorkspace, LayoutSymbolAsRootName, RemoveEmptyWorkspaces},
        layouts::paper,
    },
    core::{
        bindings::KeyEventHandler,
        config::Config,
        helpers::index_selectors,
        hooks::Hooks,
        layout::{bottom_stack, side_stack, Layout, LayoutConf},
        xconnection::XConn,
        data_types::RelativePosition,
    },
    logging_error_handler,
    xcb::new_xcb_backed_window_manager,
    xcb::new_xcb_backed_status_bar,
    Backward, Forward, Less, More, Selector,
    gen_keybindings, run_external, run_internal, map,
    draw::{Color, TextStyle}, 
};

use simplelog::{LevelFilter, SimpleLogger};

const TERM: &str = "alacritty";
const BROWSER: &str = "firefox";
const FILES: &str = "discord";
const SHOW_RUN: &str = "rofi -show run";
const MON_1: &str = "DP-2";
const MON_2: &str = "DP-4";
const BAR_HEIGHT: u32 = 78; 

fn my_layouts() -> Vec<Layout> {
    let n_main = 1;
    let ratio = 0.6;
    let follow_focus_conf = LayoutConf {
        floating: true,
        gapless: false,
        follow_focus: true,
        allow_wrapping: false,
    };

    vec![
        Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
        Layout::new("[papr]", follow_focus_conf, paper, n_main, ratio),
    ]
}

fn dynamic_workspaces<X: XConn>() -> KeyEventHandler<X> {
    create_or_switch_to_workspace(
        || {
            let options = vec!["1term", "2term", "3term", "web", "files"];
            let menu = DMenu::new("WS-SELECT: ", options, DMenuConfig::default());
            if let Ok(MenuMatch::Line(_, choice)) = menu.run(0) {
                Some(choice)
            } else {
                None
            }
        },
        my_layouts(),
    )
}

fn main() -> penrose::Result<()> {
    SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default())
        .expect("failed to init logging");

    let mut config_builder = Config::default().builder();
    let config = config_builder
        .workspaces(vec!["[1]", "[2]", "[3]", "[4]"])
        .layouts(my_layouts())
        .bar_height(BAR_HEIGHT)
        .top_bar(true)
        .build()
        .unwrap();

    let sp = Scratchpad::new(TERM, 0.8, 0.8);

    let sb = new_xcb_backed_status_bar(BAR_HEIGHT as usize, &TextStyle {
        font: "Hack".to_string(), 
        point_size: 12, 
        fg: Color::new_from_hex(0xffffff), 
        bg: Some(Color::new_from_hex(0x000000)), 
        padding: (2.0, 2.0), 
    }, Color::new_from_hex(0x123456), Color::new_from_hex(0x654321), config.workspaces().clone())?;

    let hooks: Hooks<_> = vec![
        LayoutSymbolAsRootName::new(),
        RemoveEmptyWorkspaces::new(config.workspaces().clone()),
        DefaultWorkspace::new("1term", "[side]", vec![TERM]),
        DefaultWorkspace::new("2term", "[botm]", vec![TERM, TERM]),
        DefaultWorkspace::new("3term", "[side]", vec![TERM, TERM, TERM]),
        DefaultWorkspace::new("web", "[papr]", vec![BROWSER]),
        DefaultWorkspace::new("files", "[botm]", vec![FILES]),
        Box::new(sb),
        sp.get_hook(),
    ];

    let key_bindings = gen_keybindings! {
        // Program launch
        "M-semicolon" => run_external!(SHOW_RUN);
        "M-Return" => run_external!(TERM);

        // client management
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-f" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
        "M-S-q" => run_internal!(kill_client);
        "M-slash" => sp.toggle();

        // workspace management
        "M-w" => dynamic_workspaces();
        "M-Tab" => run_internal!(toggle_workspace);
        "M-bracketleft" => run_internal!(cycle_screen, Forward);
        "M-bracketright" => run_internal!(cycle_screen, Backward);
        "M-S-bracketleft" => run_internal!(drag_workspace, Forward);
        "M-S-bracketright" => run_internal!(drag_workspace, Backward);

        // Layout management
        "M-grave" => run_internal!(cycle_layout, Forward);
        "M-S-grave" => run_internal!(cycle_layout, Backward);
        "M-A-Up" => run_internal!(update_max_main, More);
        "M-A-Down" => run_internal!(update_max_main, Less);
        "M-A-Right" => run_internal!(update_main_ratio, More);
        "M-A-Left" => run_internal!(update_main_ratio, Less);

        // Screen management
        "M-A-s" => run_internal!(detect_screens);
        "M-A-Escape" => run_internal!(exit);

        // setting up bindings for 6 possible workspaces
        refmap [config.ws_range()] in {
            "M-{}" => focus_workspace [index_selectors(config.workspaces().len())];
            "M-S-{}" => client_to_workspace [index_selectors(config.workspaces().len())];
        };
    };
    
    let sb = new_xcb_backed_status_bar(BAR_HEIGHT as usize, &TextStyle {
        font: "hack".to_string(), 
        point_size: 12, 
        fg: Color::new_from_hex(0xffffff), 
        bg: Some(Color::new_from_hex(0x000000)), 
        padding: (2.0, 2.0), 
    }, Color::new_from_hex(0x123456), Color::new_from_hex(0x654321), config.workspaces().clone())?;


    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;
    update_monitors_via_xrandr(MON_2, MON_1, RelativePosition::Left)?;
    wm.grab_keys_and_run(key_bindings, map!{})
}
