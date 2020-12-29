# FAQ

- [My graphs look kinda weird with some extra dots, how do I fix this?](#braille-support)

<h3 name="braille-support">
My graphs look kinda weird with some extra dots, how do I fix this?
</h3>

![example_image](https://user-images.githubusercontent.com/14301439/100946236-2db2f480-34c0-11eb-9f32-41202a8fe6e2.png)

You'll have to make sure you have proper braille font support. For example, for Arch, you may have to
install ttf-ubraille and/or properly set it up for your terminal.

- [Why can't I see all my processes/process usage on macOS?]

You may have to run the program with elevated privileges - i.e. `sudo btm`. If you don't like doing
this repeatedly, you can manually force it to run with root each time - for example, following
[this related guide from the htop wiki](https://github.com/hishamhm/htop/wiki/macOS:-run-without-sudo).
