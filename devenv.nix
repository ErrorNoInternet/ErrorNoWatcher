{
  pkgs,
  ...
}:

{
  packages = with pkgs; [
    git
    jujutsu

    luajit
    openssl
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
  };
}
