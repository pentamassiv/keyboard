pub const FALLBACK_LAYOUT_NAME: &str = "FALLBACK";
pub const FALLBACK_VIEW_NAME: &str = "base";

// Some of the charachter within the layout definition need to be escaped
pub const FALLBACK_LAYOUT: &str = "---
views:
    base:
        - \"q w e r t y u i o p\"
        - \"a s d f g h j k l\"
        - \"Shift_L_base z x c v b n m BackSpace\"
        - \"show_numbers show_symbols space . Return\"
    caps:
        - \"Q W E R T Y U I O P\"
        - \"A S D F G H J K L\"
        - \"Shift_L_caps Z X C V B N M BackSpace\"
        - \"show_numbers show_symbols space . Return\"
    numbers:
        - \"1 2 3 4 5 6 7 8 9 0\"
        - \"@ # $ % & - _ + ( )\"
        - \"show_symbols , \\\" ' : ; ! ? BackSpace\"
        - \"show_letters show_symbols space . Return\"
    symbols:
        - \"~ ` | · √ π τ ÷ × ¶\"
        - \"© ® £ € ¥ ^ ° * { }\"
        - \"show_numbers_from_symbols \\\\ / < > = [ ] BackSpace\"
        - \"show_letters show_numbers space . Return\"

buttons:
    Shift_L_base:
        actions:
            short_press:
                - temporarily_switch_view: caps
            long_press:
                - switch_view: caps
        key_display:
            image: key-shift.svg
        outline: standard
    Shift_L_caps:
        actions:
            short_press:
                - switch_view: base
            long_press:
                - switch_view: caps
        key_display:
            image: key-shift.svg
        outline: standard
        styles:
            - locked
    BackSpace:
        actions:
            short_press:
                - enter_keycode: BackSpace
            long_press:
                - toggle_keycode: BackSpace
        key_display:
            image: edit-clear-symbolic.svg
        outline: double
    show_numbers:
        actions:
            short_press:
                - switch_view: numbers
            long_press:
                - switch_view: numbers
        outline: standard
        key_display:
            text: 123
    show_numbers_from_symbols:
        actions:
            short_press:
                - switch_view: numbers
            long_press:
                - switch_view: numbers
        outline: standard
        key_display:
            text: 123
    show_letters:
        actions:
            short_press:
                - switch_view: base
            long_press:
                - switch_view: base
        key_display:
            text: ABC
    show_symbols:
        actions:
            short_press:
                - switch_view: symbols
            long_press:
                - switch_view: symbols
        outline: standard
        key_display:
            text: \"*/=\"
    space:
        outline: quadruple
        actions:
            short_press:
                - enter_string: \" \" 
            long_press:
            - toggle_keycode: Space
    .:
        actions:
            short_press:
                - enter_string: \".\" 
            long_press:
                - open_popup
        popup:
            - \"# @ & % \\\" '\"
            - \"( / - + ¡ ¿\"
            - \") : ; , ! ?\"
    Return:
        outline: double
        actions:
            short_press:
                - enter_keycode: Enter
            long_press:
                - toggle_keycode: Enter
        key_display:
            image: key-enter.svg
        styles:
            - return";
