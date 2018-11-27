
pub const LOGO: &'static str = "
   ▄████████ ████████▄  ▀█████████▄
  ███    ███ ███   ▀███   ███    ███
  ███    █▀  ███    ███   ███    ███
 ▄███▄▄▄     ███    ███  ▄███▄▄▄██▀
▀▀███▀▀▀     ███    ███ ▀▀███▀▀▀██▄
  ███    █▄  ███    ███   ███    ██▄
  ███    ███ ███   ▄███   ███    ███
  ██████████ ████████▀  ▄█████████▀
";

pub const SHELL: &'static str = r"
   ______ ________   __
  / __/ // / __/ /  / /
 _\ \/ _  / _// /__/ /__
/___/_//_/___/____/____/
";

pub const WELCOME: &'static str = "Welcome to EDB. Use the command `help` or `?` to display the help message.";
// run - Function to run, and the function parameters
// The Help Message
pub const HELP: &'static str = "
    Hello, I am the help message!
    You may display me using `help` or `?`
    If you need help with a specific command, use 'COMMAND help' like this: `step help` or `step?`

    Available Commands:
    help - Display this message
    clear - clear the terminal
    set - Set the parameters for the function that will be debugged
    run - Run a contract/function to debug
    reset - Reset to the first breakpoint
    chain - Chain the previous transaction into another, preserving the state trie
    step - Step a line of execution
    next - Go to the next breakpoint
    break - Set a breakpoint
    quit - use `quit` or `exit` to escape the shell
";
