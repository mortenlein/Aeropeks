import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Camera, WifiOff } from "lucide-react";

export function CameraPopover() {
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<number | null>(null);

  const fetchSnapshot = async () => {
    try {
      const b64 = await invoke<string>("get_ha_camera_snapshot");
      setImageSrc(`data:image/jpeg;base64,${b64}`);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    fetchSnapshot();
    intervalRef.current = window.setInterval(fetchSnapshot, 2000);
    return () => {
      if (intervalRef.current !== null) window.clearInterval(intervalRef.current);
    };
  }, []);

  return (
    <div className="camera-popover">
      <div className="camera-popover-header">
        <Camera size={12} />
        <span>Garage</span>
        <span className="camera-live-badge">
          <span className="camera-live-dot" />
          LIVE
        </span>
      </div>
      <div className="camera-feed-frame">
        {imageSrc ? (
          <img src={imageSrc} alt="Garage" className="camera-feed-img" />
        ) : error ? (
          <div className="camera-feed-error">
            <WifiOff size={18} />
            <span>Unavailable</span>
          </div>
        ) : (
          <div className="camera-feed-loading">
            <div className="camera-spinner" />
          </div>
        )}
      </div>
    </div>
  );
}
