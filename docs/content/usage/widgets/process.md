# Process Widget

The process widget displays a table containing information regarding a running process, along with sorting,
searching, and process control features.

<figure>
    <img src="../../../assets/screenshots/process/process_default.webp" alt="A picture of an expanded process widget by default."/>
</figure>

## Features

The process widget has three main components:

- The main process table
- The search sub-widget (opened with ++ctrl+f++ or ++slash++)
- The sort menu sub-widget (opened with ++s++ or ++f6++)

By default, the main process table displays the following information for each process:

- PID
- Name of the process
- CPU use percentage (note this is averaged out per available thread by default)
- Memory use percentage
- Disk reads per second
- Disk writes per second
- Total amount read from disk
- Total amount written from disk
- User
- Process state
- Process uptime

  <!-- 2-space indent here because mdx_truly_sane_lists interferes, see https://github.com/squidfunk/mkdocs-material/discussions/3763#discussioncomment-2833731 -->
  !!! info indent

      On Windows, the I/O counters will report _all_ reads/writes, not just disk. See
      [here](https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html#method.disk_usage)
      for more details.

With the feature flag (`--disable_gpu` on Linux/Windows to disable) and gpu process columns enabled in the configuration:

- GPU memory use percentage
- GPU core utilization percentage

See [the processes configuration page](../../configuration/config-file/processes.md) on how to customize which columns
are shown.

### Sorting

The table can be sorted by clicking on the table headers, which will either sort the table by that column, or if already
sorting by that column, reverse the sorting order.

Alternatively, one can sort using the sort menu sub-widget, which is brought up using ++s++ or ++f6++, and can be controlled by arrow keys or the mouse.

<figure>
    <img src="../../../assets/screenshots/process/process_sort_menu.webp" alt="A picture of an expanded process widget with the sort menu open."/>
</figure>

### Grouping

Pressing ++tab++ in the table will group entries with the same name together. The PID column will be replaced with the number of entries in each group, and usage
is added together when displayed.

<figure>
    <img src="../../../assets/screenshots/process/process_grouped.webp" alt="A picture of grouped mode in a process widget."/>
</figure>

Note that the process state and user columns are disabled in this mode.

### Process termination

Pressing ++d+d++ or ++f9++ will allow you to terminate the currently selected process/process group. On Unix-like
operating systems, you are also able to control which specific signals to send (e.g. `SIGKILL`, `SIGTERM`).

<figure>
    <img src="../../../assets/screenshots/process/process_kill_linux.webp" alt="A picture of the process kill menu on Linux."/>
    <figcaption><sub>The process termination menu on Linux</sub></figcaption>
</figure>

If you're on Windows, or if the `disable_advanced_kill` flag is set in the options or command-line, then a simpler termination
screen will be shown to confirm whether you want to kill that process/process group.

<figure>
    <img src="../../../assets/screenshots/process/process_kill_simple.webp" alt="A picture of the process kill menu on Windows."/>
    <figcaption><sub>The process termination menu on Windows</sub></figcaption>
</figure>

### Tree mode

Pressing ++t++ or ++f5++ in the table toggles tree mode in the process widget, displaying processes in regard to their parent-child process relationships.

<figure>
    <img src="../../../assets/screenshots/process/process_tree.webp" alt="A picture of tree mode in a process widget."/>
</figure>

A process in tree mode can also be "collapsed", hiding its children and any descendants, using either the ++minus++ or ++plus++ keys, or double-clicking on an entry.

Lastly, note that in tree mode, processes cannot be grouped together due to the behaviour of the two modes somewhat clashing.

### Full command

You can show the full command instead of just the process name by pressing ++P++.

<figure>
    <img src="../../../assets/screenshots/process/process_full.webp" alt="A picture of a process widget using full commands."/>
</figure>

### Search

Pressing ++slash++ or ++ctrl+f++ will open up the search sub-widget. By default, just typing in something will search by the process name.

<figure>
    <img src="../../../assets/screenshots/process/search/search.webp" alt="A picture of searching for a process with a simple search."/>
</figure>

This search can be further enhanced by matching by case, matching the entire word, or by regex.

<figure>
    <img src="../../../assets/screenshots/process/search/regex.webp" alt="A picture of searching for a process with a search condition that uses regex."/>
</figure>

We are able to also search for multiple things/conditions.

<figure>
    <img src="../../../assets/screenshots/process/search/or.webp" alt="A picture of searching for a process with a search condition that uses the or operator."/>
</figure>

And if our search uses a keyword, we need to use quotation marks around the term to properly search it.

<figure>
    <img src="../../../assets/screenshots/process/search/quotes.webp" alt="A picture of searching for a process with a search condition that needs quotation marks."/>
</figure>

Lastly, we can refine our search even further based on the other columns, like PID, CPU usage, etc., as well as grouping together conditions.

<figure>
    <img src="../../../assets/screenshots/process/search/cpu.webp" alt="A picture of searching for a process with a search condition that uses the CPU keyword."/>
</figure>

You can also paste search queries (e.g. ++shift+insert++, ++ctrl+shift+v++).

#### Keywords

Note all keywords are case-insensitive. To search for a process/command that collides with a keyword, surround the term with quotes (e.x. `"cpu"`).

| Keywords                        | Example                               | Description                                                                      |
| ------------------------------- | ------------------------------------- | -------------------------------------------------------------------------------- |
|                                 | `btm`                                 | Matches by process or command name; supports regex                               |
| `pid`                           | `pid=1044`                            | Matches by PID; supports regex                                                   |
| `cpu` <br/> `cpu%`              | `cpu > 0.5`                           | Matches the CPU column; supports comparison operators                            |
| `memb`                          | `memb > 1000 b`                       | Matches the memory column in terms of bytes; supports comparison operators       |
| `mem` <br/> `mem%`              | `mem < 0.5`                           | Matches the memory column in terms of percent; supports comparison operators     |
| `read` <br/> `r/s` <br/> `rps`  | `read = 1 mb`                         | Matches the read/s column in terms of bytes; supports comparison operators       |
| `write` <br/> `w/s` <br/> `wps` | `write >= 1 kb`                       | Matches the write/s column in terms of bytes; supports comparison operators      |
| `tread` <br/> `t.read`          | `tread <= 1024 gb`                    | Matches he total read column in terms of bytes; supports comparison operators    |
| `twrite` <br/> `t.write`        | `twrite > 1024 tb`                    | Matches the total write column in terms of bytes; supports comparison operators  |
| `user`                          | `user=root`                           | Matches by user; supports regex                                                  |
| `state`                         | `state=running`                       | Matches by state; supports regex                                                 |
| `()`                            | `(<COND 1> AND <COND 2>) OR <COND 3>` | Group together a condition                                                       |
| `gmem`                          | `gmem > 1000 b`                       | Matches the gpu memory column in terms of bytes; supports comparison operators   |
| `gmem%`                         | `gmem% < 0.5`                         | Matches the gpu memory column in terms of percent; supports comparison operators |
| `gpu%`                          | `gpu% > 0`                            | Matches the gpu usage column in terms of percent; supports comparison operators  |

#### Comparison operators

| Keywords | Description                                                    |
| -------- | -------------------------------------------------------------- |
| `=`      | Checks if the values are equal                                 |
| `>`      | Checks if the left value is strictly greater than the right    |
| `<`      | Checks if the left value is strictly less than the right       |
| `>=`     | Checks if the left value is greater than or equal to the right |
| `<=`     | Checks if the left value is less than or equal to the right    |

#### Logical operators

Note all operators are case-insensitive, and the `and` operator takes precedence over the `or` operator.

| Keywords                             | Usage                                                                          | Description                                         |
| ------------------------------------ | ------------------------------------------------------------------------------ | --------------------------------------------------- |
| `and` <br/> `&&` <br/> `<Space>`     | `<COND 1> and <COND 2>` <br/> `<COND 1> && <COND 2>` <br/> `<COND 1> <COND 2>` | Requires both conditions to be true to match        |
| `or` <br/> <code>&#124;&#124;</code> | `<COND 1> or <COND 2>` <br/> `<COND 1> &#124;&#124; <COND 2>`                  | Requires at least one condition to be true to match |

#### Units

All units are case-insensitive.

| Keywords | Description |
| -------- | ----------- |
| `B`      | Bytes       |
| `KB`     | Kilobytes   |
| `MB`     | Megabytes   |
| `GB`     | Gigabytes   |
| `TB`     | Terabytes   |
| `KiB`    | Kibibytes   |
| `MiB`    | Mebibytes   |
| `GiB`    | Gibibytes   |
| `TiB`    | Tebibytes   |

## Key bindings

Note that key bindings are generally case-sensitive.

### Process table

| Binding                | Action                                                           |
| ---------------------- | ---------------------------------------------------------------- |
| ++up++ , ++k++         | Move up within a widget                                          |
| ++down++ , ++j++       | Move down within a widget                                        |
| ++g+g++ , ++home++     | Jump to the first entry in the table                             |
| ++G++ , ++end++        | Jump to the last entry in the table                              |
| ++d+d++ , ++f9++       | Send a kill signal to the selected process                       |
| ++c++                  | Sort by CPU usage, press again to reverse sorting order          |
| ++m++                  | Sort by memory usage, press again to reverse sorting order       |
| ++p++                  | Sort by PID name, press again to reverse sorting order           |
| ++n++                  | Sort by process name, press again to reverse sorting order       |
| ++tab++                | Toggle grouping processes with the same name                     |
| ++P++                  | Toggle between showing the full command or just the process name |
| ++ctrl+f++ , ++slash++ | Toggle showing the search sub-widget                             |
| ++s++ , ++f6++         | Toggle showing the sort sub-widget                               |
| ++I++                  | Invert the current sort                                          |
| ++"%"++                | Toggle between values and percentages for memory usage           |
| ++t++ , ++f5++         | Toggle tree mode                                                 |
| ++M++                  | Sort by gpu memory usage, press again to reverse sorting order   |
| ++C++                  | Sort by gpu usage, press again to reverse sorting order          |

### Sort sub-widget

| Binding            | Action                                |
| ------------------ | ------------------------------------- |
| ++up++ , ++k++     | Move up within a widget               |
| ++down++ , ++j++   | Move down within a widget             |
| ++g+g++ , ++home++ | Jump to the first entry in the table  |
| ++G++ , ++end++    | Jump to the last entry in the table   |
| ++esc++            | Close the sort sub-widget             |
| ++enter++          | Sorts the corresponding process table |

### Search sub-widget

| Binding                               | Action                                       |
| ------------------------------------- | -------------------------------------------- |
| ++left++ <br/> ++h++ <br/> ++alt+h++  | Moves the cursor left                        |
| ++right++ <br/> ++l++ <br/> ++alt+l++ | Moves the cursor right                       |
| ++esc++                               | Close the search widget (retains the filter) |
| ++ctrl+a++                            | Skip to the start of the search query        |
| ++ctrl+e++                            | Skip to the end of the search query          |
| ++ctrl+u++                            | Clear the current search query               |
| ++ctrl+w++                            | Delete a word behind the cursor              |
| ++ctrl+h++                            | Delete the character behind the cursor       |
| ++backspace++                         | Delete the character behind the cursor       |
| ++delete++                            | Delete the character at the cursor           |
| ++alt+c++ , ++f1++                    | Toggle matching case                         |
| ++alt+w++ , ++f2++                    | Toggle matching the entire word              |
| ++alt+r++ , ++f3++                    | Toggle using regex                           |

## Mouse bindings

### Process table

| Binding      | Action                                                                                                                                                              |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ++"Scroll"++ | Selects a CPU thread/average to show in the graph                                                                                                                   |
| ++lbutton++  | Table header: Sorts/reverse sorts the table by the column <br/> Table entry: Selects an entry in the table, if in tree mode, collapses/expands the entry's children |

### Sort sub-widget

| Binding     | Action                        |
| ----------- | ----------------------------- |
| ++lbutton++ | Selects an entry in the table |
