# ARCHIVO INTERNO DE KORUNIX.
#
# ¿Qué es?
# Aquí Korunix aprende qué significa cada canal de actualizaciones.
#
# ¿Para qué sirve?
# Un equipo solo guarda una elección sencilla: "stable" o "unstable".
# Este archivo explica qué fuentes corresponden a cada elección y qué texto
# debe mostrarse a una persona.
#
# ¿Debes cambiarlo para elegir un canal?
# No. La elección de cada computadora vive en su archivo dentro de configuracion/equipos/.
# Este archivo solo cambia cuando Korunix cambia la definición del producto.
#
# Partes importantes:
# - default: canal que recibe una instalación nueva si todavía no eligió otro.
# - channels.stable: definición del canal Estable.
# - channels.unstable: definición del canal Inestable.
# - label/description: nombre y explicación que ve una persona.
# - nixpkgs_ref/aagl_ref: fuentes técnicas que Korunix usa por debajo.
{
  schemaVersion = 1;
  default = "stable";

  channels = {
    stable = {
      label = "Estable";
      description = "Prioriza estabilidad y cambios menos frecuentes.";
      nixpkgs_ref = "nixos-26.05";
      aagl_ref = "release-26.05";
      label_en = "Stable";
      description_en = "Prioritizes stability and less frequent changes.";
      label_hu = "Stabil";
      description_hu = "A stabilitást és a ritkább változásokat részesíti előnyben.";
    };

    unstable = {
      label = "Inestable";
      description = "Prioriza software reciente y cambios más frecuentes.";
      nixpkgs_ref = "nixos-unstable";
      aagl_ref = "main";
      label_en = "Unstable";
      description_en = "Prioritizes newer software and more frequent changes.";
      label_hu = "Instabil";
      description_hu = "Az újabb szoftvereket és a gyakoribb változásokat részesíti előnyben.";
    };
  };
}
