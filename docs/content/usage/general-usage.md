# General Usage

You can run bottom with:

```bash
btm
```

For help regarding the command-line options, use:

```bash
# For a simple overview of flags
btm -h

# For more details
btm --help
```

You can also see keybinds and basic usage details in bottom by pressing ++question++, which will open a help menu.

## Features

### Expansion

By default, bottom is somewhat like a dashboard - a bunch of different widgets, all showing different things, and they all cram together to fit into one terminal.

If you instead just want to see _one_ widget - maybe you want to look at a graph in more detail, for example - you can "expand" the currently selected
widget using the ++e++ key, which will hide all other widgets and make that widget take up all available terminal space.

You can leave this state by either pressing ++e++ again or pressing ++esc++.

### Widget selection

To allow for widget-specific keybindings and expansion, there is the idea of _widget selection_ in bottom, where you can focus on a specific widget to work with it.
This can be done with the mouse (just click on the widget of interest) or keyboard (ex: ++ctrl+"Direction"++, see [Key bindings](#key-bindings) for alternatives).

## Key bindings

These are global or common keyboard shortcuts for the application, which you can see in-app through the ++question++ shortcut.
Note that key bindings are generally case-sensitive.

| Binding                                                      | Action                                                       |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| ++q++ , ++ctrl+c++                                           | Quit                                                         |
| ++esc++                                                      | Close dialog windows, search, widgets, or exit expanded mode |
| ++ctrl+r++                                                   | Reset display and any collected data                         |
| ++f++                                                        | Freeze/unfreeze updating with new data                       |
| ++question++                                                 | Open help menu                                               |
| ++e++                                                        | Toggle expanding the currently selected widget               |
| ++ctrl+up++ <br/> ++shift+up++ <br/> ++K++ <br/> ++W++       | Select the widget above                                      |
| ++ctrl+down++ <br/> ++shift+down++ <br/> ++J++ <br/> ++S++   | Select the widget below                                      |
| ++ctrl+left++ <br/> ++shift+left++ <br/> ++H++ <br/> ++A++   | Select the widget on the left                                |
| ++ctrl+right++ <br/> ++shift+right++ <br/> ++L++ <br/> ++D++ | Select the widget on the right                               |
| ++up++ , ++k++                                               | Move up within a widget                                      |
| ++down++ , ++j++                                             | Move down within a widget                                    |
| ++left++ <br/> ++h++ <br/> ++alt+h++                         | Move left within a widget                                    |
| ++right++ <br/> ++l++ <br/> ++alt+l++                        | Move right within a widget                                   |
| ++g+g++ , ++home++                                           | Jump to the first entry                                      |
| ++G++ , ++end++                                              | Jump to the last entry                                       |
| ++page-up++ , ++page-down++                                  | Scroll up/down a table by a page                             |
| ++ctrl+u++                                                   | Scroll up a table by half a page                             |
| ++ctrl+d++                                                   | Scroll down a table by half a page                           |

## Mouse bindings

| Binding     | Action             |
| ----------- | ------------------ |
| ++lbutton++ | Selects the widget |
