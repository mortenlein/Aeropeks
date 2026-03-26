import { Cloud, Sun, CloudRain, CloudLightning, Snowflake, Wind, CloudSun, CloudDrizzle } from "lucide-react";

interface HourlyForecast {
  time: string;
  temp: number;
  symbol: string;
  precip: number;
}

interface DailyForecast {
  date: string;
  temp_min: number;
  temp_max: number;
  symbol: string;
}

interface WeatherDetailed {
  temp: number;
  symbol: string;
  precip: number;
  place_name: string;
  hourly: HourlyForecast[];
  daily: DailyForecast[];
}

interface Props {
  data: WeatherDetailed;
  onClose: () => void;
}

const getWeatherIcon = (symbol: string, size = 20) => {
  const s = symbol.toLowerCase();
  if (s.includes("clearsky")) return <Sun size={size} className="weather-icon-sun" />;
  if (s.includes("fair") || s.includes("partlycloudy")) return <CloudSun size={size} className="weather-icon-cloud" />;
  if (s.includes("cloudy")) return <Cloud size={size} className="weather-icon-cloud" />;
  if (s.includes("rain") && s.includes("heavy")) return <CloudRain size={size} className="weather-icon-rain" />;
  if (s.includes("rain") || s.includes("drizzle")) return <CloudDrizzle size={size} className="weather-icon-rain" />;
  if (s.includes("snow")) return <Snowflake size={size} className="weather-icon-snow" />;
  if (s.includes("thunder")) return <CloudLightning size={size} className="weather-icon-thunder" />;
  if (s.includes("fog")) return <Wind size={size} className="weather-icon-fog" />;
  return <Cloud size={size} />;
};

const formatTime = (iso: string) => {
  const date = new Date(iso);
  return date.getHours().toString().padStart(2, '0') + ':00';
};

const formatDate = (iso: string) => {
  const date = new Date(iso);
  return date.toLocaleDateString('nb-NO', { weekday: 'short', day: 'numeric', month: 'short' });
};

export function WeatherPopover({ data, onClose }: Props) {
  return (
    <div className="weather-popover-container">
      <div className="weather-popover-header">
        <div className="weather-current-main">
          <div className="weather-current-temp">{Math.round(data.temp)}°</div>
          <div className="weather-current-info">
            <div className="weather-place-name">{data.place_name}</div>
            <div className="weather-symbol-label">
              {getWeatherIcon(data.symbol, 18)}
              <span>{data.symbol.replace(/_/g, ' ')}</span>
            </div>
          </div>
        </div>
        <button className="weather-close-btn" onClick={onClose}>×</button>
      </div>

      <div className="weather-section-label">Hourly Forecast</div>
      <div className="weather-hourly-list">
        {data.hourly.slice(0, 24).map((h, i) => (
          <div key={i} className="weather-hourly-item">
            <div className="hourly-time">{i === 0 ? "Now" : formatTime(h.time)}</div>
            <div className="hourly-icon">{getWeatherIcon(h.symbol, 22)}</div>
            <div className="hourly-temp">{Math.round(h.temp)}°</div>
            <div className="hourly-precip">{h.precip > 0 ? `${h.precip.toFixed(1)}mm` : "-"}</div>
          </div>
        ))}
      </div>

      <div className="weather-section-label">Next 7 Days</div>
      <div className="weather-daily-list">
        {data.daily.map((d, i) => (
          <div key={i} className="weather-daily-item">
            <div className="daily-date">{i === 0 ? "Today" : formatDate(d.date)}</div>
            <div className="daily-icon">{getWeatherIcon(d.symbol, 20)}</div>
            <div className="daily-temps">
              <span className="temp-max">{Math.round(d.temp_max)}°</span>
              <span className="temp-min">{Math.round(d.temp_min)}°</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
