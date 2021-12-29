// TODO: Add X11rb backed status bar
// TODO: Electron applications appear to cause issues with penrose
// See if making them float fixes this.
#[macro_use]
extern crate penrose;
use penrose_config::{
    hooks::StartupScript,
    layouts::stack_sides,
};

use penrose::{
    contrib::{
        actions::{update_monitors_via_xrandr},
        extensions::{Scratchpad},
        hooks::{LayoutSymbolAsRootName, RemoveEmptyWorkspaces},
    },
    core::{
        config::Config,
        helpers::index_selectors,
        layout::{bottom_stack, side_stack, Layout, LayoutConf},
        data_types::RelativePosition,
    },
    logging_error_handler,
    x11rb::new_x11rb_rust_backed_window_manager,
    x11rb::X11rbHooks,
    Backward, Forward, Less, More, Selector,
};

use simplelog::{LevelFilter, SimpleLogger};

const TERM: &str = "alacritty";
const SHOW_RUN: &str = "rofi -show run";
const MON_1: &str = "DP-2";
const MON_2: &str = "DP-4";

fn my_layouts() -> Vec<Layout> {
    let n_main = 1;
    let ratio = 0.6;

    vec![
        Layout::new("[sides]", LayoutConf::default(), stack_sides, 0u32, 0f32),
        Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
    ]
}


fn main() -> penrose::Result<()> {
    SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default()).expect("failed to init logging");

    let mut config_builder = Config::default().builder();
    let config = config_builder
        .workspaces(vec!["[1]", "[2]", "[3]", "[4]"])
        .layouts(my_layouts())
        .build()
        .unwrap();

    let sp = Scratchpad::new(TERM, 0.8, 0.8);

    let hooks: X11rbHooks = vec![
        LayoutSymbolAsRootName::new(),
        RemoveEmptyWorkspaces::new(config.workspaces().clone()),
        Box::new(StartupScript::new("/usr/local/scripts/penrose-config-setup.sh")),
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
        
        // Set volume
        "XF86AudioLowerVolume" => run_external!("pactl set-sink-volume @DEFAULT_SINK@ -5%");
        "XF86AudioRaiseVolume" => run_external!("pactl set-sink-volume @DEFAULT_SINK@ +5%");
        "XF86AudioMute" => run_external!("pactl set-sink-mute @DEFAULT_SINK@ toggle");

        // Screen management
        "M-A-s" => run_internal!(detect_screens);
        "M-A-Escape" => run_internal!(exit);

        map: {"1", "2", "3", "4"} to index_selectors(4) => {
            "M-{}" => focus_workspace (REF);
            "M-S-{}" => client_to_workspace (REF);
        };
    };
    

    let mut wm = new_x11rb_rust_backed_window_manager(config, hooks, logging_error_handler())?;
    update_monitors_via_xrandr(MON_2, MON_1, RelativePosition::Left)?;
    wm.grab_keys_and_run(key_bindings, map!{})
}
