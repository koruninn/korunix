{pkgs, ...}: let
  commandSuggest = pkgs.writeShellApplication {
    name = "korunix-command-suggest";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.findutils
      pkgs.gawk
    ];
    text = ''
      needle="''${1:-}"
      [ -n "$needle" ] || exit 1

      {
        # Fish puede pasar por stdin sus comandos internos.
        cat

        # Añadimos todos los ejecutables disponibles en PATH.
        IFS=':' read -ra dirs <<< "''${PATH:-}"
        for dir in "''${dirs[@]}"; do
          [ -d "$dir" ] || continue
          find -L "$dir" \
            -maxdepth 1 \
            -mindepth 1 \
            -type f \
            -executable \
            -printf '%f\n' 2>/dev/null || true
        done
      } | sort -u | awk -v needle="$needle" '
        function min2(a, b) { return a < b ? a : b }
        function min3(a, b, c) { return min2(min2(a, b), c) }

        # Distancia Damerau-Levenshtein: una transposición como gti -> git
        # cuenta como un solo error.
        function distance(a, b,    n, m, i, j, cost, value, ca, cb, d) {
          delete d
          n = length(a)
          m = length(b)

          for (i = 0; i <= n; i++) d[i, 0] = i
          for (j = 0; j <= m; j++) d[0, j] = j

          for (i = 1; i <= n; i++) {
            ca = substr(a, i, 1)
            for (j = 1; j <= m; j++) {
              cb = substr(b, j, 1)
              cost = (ca == cb ? 0 : 1)
              value = min3(
                d[i - 1, j] + 1,
                d[i, j - 1] + 1,
                d[i - 1, j - 1] + cost
              )

              if (i > 1 && j > 1 &&
                  ca == substr(b, j - 1, 1) &&
                  substr(a, i - 1, 1) == cb) {
                value = min2(value, d[i - 2, j - 2] + 1)
              }

              d[i, j] = value
            }
          }

          return d[n, m]
        }

        {
          candidate = $0
          if (candidate == "" || candidate == needle) next

          delta = length(candidate) - length(needle)
          if (delta < 0) delta = -delta
          if (delta > 3) next

          dist = distance(needle, candidate)
          score = dist

          # En empates preferimos candidatos que empiecen igual.
          if (substr(candidate, 1, 1) == substr(needle, 1, 1)) score -= 0.20

          if (!found || score < best_score ||
              (score == best_score && length(candidate) < length(best))) {
            found = 1
            best = candidate
            best_dist = dist
            best_score = score
          }
        }

        END {
          n = length(needle)
          limit = (n <= 2 ? 1 : (n <= 5 ? 1 : (n <= 9 ? 2 : 3)))
          if (found && best_dist <= limit) print best
        }
      '
    '';
  };
in {
  # Habilita Fish y lo deja como shell de inicio por defecto para los usuarios
  # normales que no definan otro shell explícitamente.
  programs.fish = {
    enable = true;
    interactiveShellInit = ''
      set -g fish_greeting ""

      function fish_command_not_found
        set -l failed $argv[1]

        # No intentamos corregir rutas escritas explícitamente.
        if string match --quiet '*/*' -- $failed
          printf 'fish: no se encontró «%s»\n' $failed >&2
          return 127
        end

        set -l suggestion (builtin --names | korunix-command-suggest $failed)

        if test -z "$suggestion"
          printf 'fish: comando desconocido «%s»\n' $failed >&2
          return 127
        end

        printf '¿Corregir «%s» a «%s»? [y/n] ' $failed $suggestion
        read --nchars 1 --local answer
        echo

        switch $answer
          case y Y
            $suggestion $argv[2..]
            return $status
          case '*'
            return 127
        end
      end

      fastfetch --config /etc/fastfetch/config.jsonc
      echo
    '';
  };

  environment.systemPackages = [
    commandSuggest
  ];

  users.defaultUserShell = pkgs.fish;
}
