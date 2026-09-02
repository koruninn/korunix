{
  config,
  inputs,
  korunixContext,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix;
  channelCatalog = import ./canales.nix;

  # Este contrato pertenece a la generación activa. El motor puede leerlo sin
  # volver a evaluar el flake cada vez que se abre una página.
  personaEntries = builtins.readDir korunixContext.personasPath;

  personaNames =
    lib.sort builtins.lessThan
    (builtins.attrNames (
      lib.filterAttrs
      (
        name: type:
          type
          == "regular"
          && lib.hasSuffix ".nix" name
      )
      personaEntries
    ));

  personaIds = map (lib.removeSuffix ".nix") personaNames;

  profileFor = userId:
    import (korunixContext.personasPath + "/${userId}.nix");

  profiles = lib.genAttrs personaIds profileFor;

  assignedFor = userId:
    lib.elem userId cfg.users;

  settingsFor = userId:
    cfg.userSettings.${userId} or {};

  accountNameFor = userId: let
    profile = profiles.${userId};
    settings = settingsFor userId;
    localName = settings.accountName or null;
  in
    if localName != null
    then localName
    else profile.accountName or userId;

  homeDirectoryFor = userId: let
    settings = settingsFor userId;
    localHome = settings.homeDirectory or null;
  in
    if localHome != null
    then localHome
    else "/home/${accountNameFor userId}";

  administratorFor = userId:
    (settingsFor userId).administrator or false;

  deferredCapabilitiesFor = userId:
    (settingsFor userId).deferredCapabilities or [];

  requestedCapabilitiesFor = userId:
    profiles.${userId}.capabilities or [];

  enabledCapabilitiesFor = userId:
    lib.filter
    (
      capability:
        !(lib.elem capability (deferredCapabilitiesFor userId))
    )
    (requestedCapabilitiesFor userId);

  deferredInputMethodsFor = userId:
    (settingsFor userId).deferredInputMethods or [];

  requestedInputMethodsFor = userId:
    profiles.${userId}.inputMethods or [];

  enabledInputMethodsFor = userId:
    lib.filter
    (
      inputMethod:
        !(lib.elem inputMethod (deferredInputMethodsFor userId))
    )
    (requestedInputMethodsFor userId);

  preservedGroupsFor = userId:
    (settingsFor userId).preservedGroups or [];

  profileStateFor = userId: let
    profile = profiles.${userId};
    accountName = accountNameFor userId;
    assigned = assignedFor userId;
  in {
    id = userId;
    accountName = profile.accountName or userId;
    fullName = profile.fullName or "";
    language = profile.language or null;
    interfaceLanguage = profile.interfaceLanguage or null;
    inputMethods = requestedInputMethodsFor userId;
    capabilities = requestedCapabilitiesFor userId;

    assignedToHost = assigned;
    effectiveAccountName = accountName;
    homeDirectory = homeDirectoryFor userId;
    administrator = administratorFor userId;
    enabledCapabilities = enabledCapabilitiesFor userId;
    deferredCapabilities = deferredCapabilitiesFor userId;
    enabledInputMethods = enabledInputMethodsFor userId;
    deferredInputMethods = deferredInputMethodsFor userId;
    preservedGroups = preservedGroupsFor userId;

    declaredGroups =
      if assigned && builtins.hasAttr accountName config.users.users
      then config.users.users.${accountName}.extraGroups or []
      else [];
  };

  profilesState = map profileStateFor personaIds;

  hostInputMethods = lib.unique (
    lib.concatMap enabledInputMethodsFor cfg.users
  );

  advancedInputMethodsEnabled = hostInputMethods != [];

  inputMethodCatalog = [
    {
      id = "chinese-pinyin";
      label = "Chino — Pinyin";
      engine = "pinyin";
    }
    {
      id = "korean-hangul";
      label = "Coreano — Hangul";
      engine = "hangul";
    }
    {
      id = "japanese-mozc";
      label = "Japonés — Mozc";
      engine = "mozc";
    }
    {
      id = "vietnamese-unikey";
      label = "Vietnamita — Unikey";
      engine = "unikey";
    }
  ];

  engineFor = inputMethod:
    (lib.findFirst
      (entry: entry.id == inputMethod)
      (throw "Korunix no conoce el método de entrada ${inputMethod}.")
      inputMethodCatalog).engine;

  fcitxDefaultLayout =
    if cfg.localization.keyboard.variant == ""
    then cfg.localization.keyboard.layout
    else
      cfg.localization.keyboard.layout
      + "-"
      + cfg.localization.keyboard.variant;

  inputMethodModel = {
    schemaVersion = 1;
    backend =
      if advancedInputMethodsEnabled
      then "fcitx5"
      else "ibus";
    launcher = "xdg-autostart";
    keyboard = {
      engine = "keyboard-" + fcitxDefaultLayout;
      layout = fcitxDefaultLayout;
    };
    catalog = inputMethodCatalog;
    people =
      map
      (userId: {
        id = userId;
        requested = requestedInputMethodsFor userId;
        deferred = deferredInputMethodsFor userId;
        effective = enabledInputMethodsFor userId;
        engines = map engineFor (enabledInputMethodsFor userId);
      })
      cfg.users;
  };

  noctaliaTranslations =
    builtins.readDir (inputs.noctalia.outPath + "/assets/translations");

  noctaliaLanguages =
    lib.sort builtins.lessThan
    (map
      (lib.removeSuffix ".json")
      (builtins.attrNames (
        lib.filterAttrs
        (
          name: type:
            type
            == "regular"
            && lib.hasSuffix ".json" name
        )
        noctaliaTranslations
      )));

  sourceEntry = path: relative: {
    path = relative;
    sha256 = builtins.hashFile "sha256" path;
  };

  personaSources =
    map
    (
      name:
        sourceEntry
        (korunixContext.personasPath + "/${name}")
        "configuracion/personas/${name}"
    )
    personaNames;

  inputMethodPackage =
    if (config.i18n.inputMethod.package or null) == null
    then ""
    else toString config.i18n.inputMethod.package;

  runtimeState = {
    schemaVersion = 1;
    kind = "korunix-runtime-state";
    hostId = cfg.hostId;

    sourceHashes = {
      host =
        sourceEntry
        korunixContext.hostFile
        "configuracion/equipos/${cfg.hostId}.nix";

      hardware =
        sourceEntry
        korunixContext.hardwareFile
        "generado/equipos/${cfg.hostId}-detectado.nix";

      channels =
        sourceEntry
        ./canales.nix
        "sistema/canales.nix";

      personas = personaSources;
    };

    channel = {
      effective = cfg.channel;
      nixosVersion = config.system.nixos.version;
      stateVersion = config.system.stateVersion;
      model = channelCatalog.channels;
    };

    hardware = {
      platform = config.nixpkgs.hostPlatform.system;
      firmware = cfg.hardware.firmware;
    };

    people = {
      hostUserIds = cfg.users;
      mutableUsers = config.users.mutableUsers;
      profiles = profilesState;
    };

    localization = {
      declared = cfg.localization;

      derived = {
        systemLocale = config.i18n.defaultLocale;
        formatLocale = config.i18n.extraLocaleSettings.LC_TIME;
        keyboard = {
          layout = config.services.xserver.xkb.layout;
          variant = config.services.xserver.xkb.variant;
          options = config.services.xserver.xkb.options;
          console = config.console.keyMap;
        };
      };

      noctaliaLanguages = noctaliaLanguages;

      inputMethod = {
        candidate = inputMethodModel;
        nixos = {
          enabled = config.i18n.inputMethod.enable;
          type = config.i18n.inputMethod.type;
          package = inputMethodPackage;
        };
      };

      catalog = {
        xkbRoot = toString pkgs.xkeyboard_config;
        tzdataRoot = toString pkgs.tzdata;
      };
    };
  };
in {
  config = lib.mkIf cfg.enable {
    environment.etc."korunix/runtime-state.json".text =
      builtins.toJSON runtimeState;
  };
}
