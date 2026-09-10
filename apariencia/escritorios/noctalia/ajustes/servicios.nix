{...}: {
  programs.noctalia.settings = {
    calendar = {
      enabled = true;
      refresh_minutes = 15;
      account.mi_google_cal = {
        type = "google";
        name = "Google Calendar";
      };
    };
  };
}
