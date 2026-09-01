# ESTE ARCHIVO SE PUEDE CAMBIAR.
#
# ¿Qué es?
# Aquí viven las preferencias que pertenecen a una persona y pueden acompañarla
# entre computadoras.
#
# Puede guardar, por ejemplo, su nombre visible, idioma, métodos de entrada,
# capacidades y avatar. No guarda contraseñas ni hashes de contraseñas.
#
# Las decisiones exclusivas de una computadora viven en configuracion/equipos/.
#
{
  # Identidad portable. accountName es la cuenta UNIX preferida, pero un host
  # puede definir una excepción local si ese nombre ya está ocupado.
  accountName = "koru";
  fullName = "André";
  # Idioma personal de sesión. No controla la interfaz de Korunix.
  language = "es";

  # null = seguir el locale actual al abrir Korunix.
  # Un código explícito cambia solo la interfaz de Korunix.
  interfaceLanguage = null;

  # Los métodos de entrada avanzados son una preferencia portable.
  # Una lista vacía significa que esta persona solo necesita el teclado normal.
  inputMethods = [];

  # Las capacidades describen lo que la persona quiere poder hacer. Un equipo
  # que temporalmente no pueda satisfacer alguna la conserva como aplazada en
  # su estado local en vez de borrar la intención del perfil.
  capabilities = [
    "android"
    "printing"
    "sunshine"
    "virtualization"
  ];

  # El avatar forma parte de la identidad portable y puede viajar dentro de un
  # bundle .korunix-profile. No contiene ninguna credencial.
  avatar =
    if builtins.pathExists ./koru.jpg
    then ./koru.jpg
    else null;
}
