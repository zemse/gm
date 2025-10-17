use std::time::Duration;

pub fn demo_text() -> &'static str {
    "Welcome to demo trial!\n\
    \n\
    This program is currently running on cloud server (not on your computer), so some features are disabled in demo mode.\n\
    \n\
    Here are few things you can try:\n\
    - Create a new account (don't send any funds to it as this key will be deleted after demo session ends).\n\
    - Try using `walletconnect` to connect with a website (e.g. sign message on https://etherscan.io/verifiedSignatures).\n\
    - Try using the `shell` to prevent passing secrets to a js script that signs a message."
}

pub const DEMO_2_DELAY: Duration = Duration::from_secs(10);

pub fn demo_text_2() -> &'static str {
    "Looks like you've been exploring gm for a while! Due to resource constraints on the server, this demo session has to be time limited.\n\
    \n\
    Install the full version of gm on your computer to:\n\
    - Create or load your accounts\n\
    - Manage assets and send transactions\n\
    - Run foundry/hardhat scripts without .env files\n\
    - TouchID on macOS\n\
    - And more!\n\
    \n\
    Instructions: https://github.com/zemse/gm\n\
    If you like this project, please consider starring it on github, it helps a lot in many ways.\n\
    \n\
    Feedback, suggestions, or questions? Feel free to chat with telegram (https://t.me/zemse) or discord @zemse"
}

pub fn demo_exit_text() -> &'static str {
    "\n\
    Thank you for your time into trying the gm demo. It means a lot to us!\n\
    \n\
    It would be awesome if you can share your demo experience with our team regarding what you liked and what can be better.\n\
    \n\
    Please share your feedback with @zemse on telegram or discord. Can't guarentee but we might have a small gift for you!\n"
}
