"""Mantener el modo claro y oscuro de Noctalia cuando uso GNOME."""
import datetime as dt
import json
import math
import pathlib
import sys
import tomllib

def section_files(config, settings):
    # Leer primero la configuración y después los ajustes guardados, como Noctalia.
    files = [config] + sorted(p for p in config.parent.glob("*.toml") if p != config) + [settings]
    location, theme = {}, {}
    for path in files:
        try:
            data = tomllib.loads(path.read_text())
            location.update(data.get("location", {}))
            theme.update(data.get("theme", {}))
        except (OSError, ValueError, TypeError):
            continue
    return location, theme

def minutes(value):
    t = dt.time.fromisoformat(value)
    return t.hour * 60 + t.minute

def solar_times(now, lat, lon):
    # Cálculo adaptado de Noctalia (MIT), day_night_schedule.cpp.
    # Véase LICENSES/third-party-notices.txt.
    y = 2 * math.pi / 365 * (now.timetuple().tm_yday - 1)
    e = 229.18 * (0.000075 + 0.001868*math.cos(y) - 0.032077*math.sin(y)
                  - 0.014615*math.cos(2*y) - 0.040849*math.sin(2*y))
    d = (0.006918 - 0.399912*math.cos(y) + 0.070257*math.sin(y)
         - 0.006758*math.cos(2*y) + 0.000907*math.sin(2*y)
         - 0.002697*math.cos(3*y) + 0.00148*math.sin(3*y))
    l = math.radians(lat)
    x = math.cos(math.radians(90.833))/(math.cos(l)*math.cos(d)) - math.tan(l)*math.tan(d)
    if x > 1:
        return None, None, "dark"
    if x < -1:
        return None, None, "light"
    h = math.degrees(math.acos(x))
    noon = 720 - 4*lon - e + now.utcoffset().total_seconds()/60
    return round(noon - 4*h) % 1440, round(noon + 4*h) % 1440, None

def resolve(config, settings, cache, now):
    loc, theme = section_files(config, settings)
    configured = theme.get("mode", "auto")
    if configured in ("dark", "light"):
        return configured
    if loc.get("custom_schedule"):
        try:
            sunrise, sunset = minutes(loc["sunrise"]), minutes(loc["sunset"])
        except (KeyError, TypeError, ValueError):
            return "light"
    else:
        lat, lon = loc.get("latitude", -12.05), loc.get("longitude", -77.04)
        try:
            cached = json.loads(cache.read_text())
            auto = bool(loc.get("auto_locate"))
            address = loc.get("address", "")
            if (auto or address) and cached.get("auto_locate") == auto and (auto or cached.get("address") == address):
                la, lo = float(cached["latitude"]), float(cached["longitude"])
                if -90 <= la <= 90 and -180 <= lo <= 180:
                    lat, lon = la, lo
        except (OSError, ValueError, TypeError, KeyError):
            pass
        sunrise, sunset, fixed = solar_times(now, float(lat), float(lon))
        if fixed:
            return fixed
    current = now.hour * 60 + now.minute
    night = (current >= sunset or current < sunrise) if sunset > sunrise else (sunset <= current < sunrise)
    return "dark" if night else "light"

if __name__ == "__main__":
    print(resolve(*(pathlib.Path(p) for p in sys.argv[1:]), dt.datetime.now().astimezone()))
