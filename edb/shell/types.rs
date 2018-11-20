
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

// The Help Message
pub const HELP: &'static str = "
    Hello, I am the help message!
    You may display me using `help` or `?`
    If you need help with a specific command, use 'COMMAND help' like this: `step help` or `step?`

    Available Commands:
    run - need the Address of deployed contract, Contract Name, Function to run, and the function parameters
    execute - Execute the contract without stopping at any breakpionts
    step - Step a line of execution
    next - Go to the next breakpoint
    break - Set a breakpoint
    memory - Display the Memory
    storage - Display the Storage
    stack - Display the Stack
    print - Print things
";
