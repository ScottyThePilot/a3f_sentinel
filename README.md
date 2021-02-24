# A3F Sentinel

A3F Sentinel (aka. "Northrop Grumman X-47B") is a really basic
discord bot for automating things in the A3F discord server.

To run it, all you need to do is clone the code and compile.
On first time running, the program will create a `config.ron`
which you will need to fill out. You will need to remove the
parentheses it adds around IDs or it won't read it properly
again for some reason.

Current commands are:
- `$ping` and `$stop` of course
- `$promote <user>`, `$demote <user>` and `$setrank <user> <rank...>` for changing user ranks
- `$assign <user> <role>`, `$unassign <user> <role>` for managing assignable roles
- `$emojidata <emoji>` for getting emojis in a form usable in `config.ron`
- `$reload` for reloading the config file
