{
  # Identificador humano de la cuenta. El archivo no guarda contraseñas: la clave
  # existente de Calamares permanece en el sistema y las nuevas credenciales se
  # gestionarán localmente mediante la capa privilegiada de Korunix.
  accountName = "koru";
  fullName = "André";
  homeDirectory = "/home/koru";
  language = "es";

  # Korunix traduce este rol a los permisos del sistema. La interfaz normal nunca
  # necesita preguntar si una persona conoce el grupo UNIX «wheel».
  administrator = true;

  # Las capacidades expresan lo que la persona necesita hacer. Los grupos UNIX
  # se derivan de ellas y no son una lista que el usuario tenga que mantener.
  capabilities = [
    "android"
    "printing"
    "sunshine"
    "virtualization"
  ];

  # El avatar pertenece a la identidad del usuario, no a Noctalia. Si todavía no
  # existe el archivo, Korunix simplemente continúa sin una imagen personalizada.
  avatar =
    if builtins.pathExists ./koru.jpg
    then ./koru.jpg
    else null;
}
