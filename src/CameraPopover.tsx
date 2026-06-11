import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Panel, Micro } from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";

export function CameraPopover({ label = "Camera" }: { label?: string }) {
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
    <Panel
      w={460}
      title={label}
      icon={<Icon name="cam" size={13} />}
      hue={HUE.red}
      style={{ right: 0 }}
      actions={
        <span style={{ display: "flex", alignItems: "center", gap: 5 }}>
          <span style={{ width: 5, height: 5, borderRadius: 999, background: HUE.red, animation: "aeroPulse 2.2s ease-in-out infinite", flexShrink: 0 }} />
          <Micro color={HUE.red} style={{ margin: 0 }}>LIVE</Micro>
        </span>
      }
    >
      <div style={{ borderRadius: T.cardR, overflow: "hidden", background: "rgba(0,0,0,0.3)", aspectRatio: "16 / 9", display: "flex", alignItems: "center", justifyContent: "center" }}>
        {imageSrc ? (
          <img src={imageSrc} alt={label} style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />
        ) : error ? (
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 8, color: T.t3 }}>
            <Icon name="cam" size={18} />
            <span style={{ fontSize: 12 }}>Unavailable</span>
          </div>
        ) : (
          <div style={{ width: 20, height: 20, borderRadius: 999, border: `2px solid ${T.t3}`, borderTopColor: "transparent", animation: "spin 0.8s linear infinite" }} />
        )}
      </div>
    </Panel>
  );
}
