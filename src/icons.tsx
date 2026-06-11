/**
 * Aeropeks icon set — Terminal Precision.
 * THE ONLY ICONS ALLOWED IN THE APP. Do not use lucide/font-awesome/segoe/fluent.
 * Source: design_handoff_aeropeks_theme/icons.js (verbatim SVG strings).
 * Render at 10-14px; color via CSS `color` on the wrapper.
 */
import type { CSSProperties } from 'react';

// Verbatim from design_handoff_aeropeks_theme/icons.js — do not edit paths.
const AERO_ICONS: Record<string, string> = {
  "close":    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M3 3l6 6M9 3l-6 6\"></path></svg>",
  "refresh":  "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M10 6a4 4 0 11-1.2-2.8M9 1v2.5H6.5\"></path></svg>",
  "search":   "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><circle cx=\"5.2\" cy=\"5.2\" r=\"3.4\"></circle><path d=\"M8 8l2.6 2.6\"></path></g></svg>",
  "prev":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M3 2.5v7\"></path><path d=\"M9.5 2.5L4.5 6l5 3.5v-7z\" fill=\"currentColor\" stroke=\"none\"></path></g></svg>",
  "next":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M9 2.5v7\"></path><path d=\"M2.5 2.5L7.5 6l-5 3.5v-7z\" fill=\"currentColor\" stroke=\"none\"></path></g></svg>",
  "pause":    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M4 2.5v7M8 2.5v7\" stroke-width=\"1.8\"></path></g></svg>",
  "play":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M3.5 2.2L9.8 6l-6.3 3.8v-7.6z\" fill=\"currentColor\" stroke=\"none\"></path></svg>",
  "wifi":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M2 5a6 6 0 018 0M3.7 7a3.6 3.6 0 014.6 0\"></path><circle cx=\"6\" cy=\"9\" r=\"0.9\" fill=\"currentColor\" stroke=\"none\"></circle></g></svg>",
  "battery":  "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"1\" y=\"3.5\" width=\"8.5\" height=\"5\" rx=\"1.2\"></rect><path d=\"M11 5.2v1.6\" stroke-width=\"1.6\"></path></g></svg>",
  "bolt":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M6.5 1L3 6.8h2.6L5.2 11 9 5.4H6.2L6.5 1z\" fill=\"currentColor\" stroke=\"none\"></path></svg>",
  "clock":    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><circle cx=\"6\" cy=\"6\" r=\"4.6\"></circle><path d=\"M6 3.6V6l1.8 1.2\"></path></g></svg>",
  "cal":      "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"1.5\" y=\"2.5\" width=\"9\" height=\"8\" rx=\"1.5\"></rect><path d=\"M1.5 5h9M4 1.2v2M8 1.2v2\"></path></g></svg>",
  "cam":      "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"1\" y=\"3.5\" width=\"7\" height=\"5.5\" rx=\"1.2\"></rect><path d=\"M8 5.5l3-1.5v4.5l-3-1.5\"></path></g></svg>",
  "term":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M2 3.5L5 6 2 8.5\"></path><path d=\"M6.5 8.5H10\"></path></g></svg>",
  "power":    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M6 1.5v4\"></path><path d=\"M3.4 3.2a4.4 4.4 0 105.2 0\"></path></g></svg>",
  "lock":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"2.5\" y=\"5\" width=\"7\" height=\"5.5\" rx=\"1.2\"></rect><path d=\"M4 5V3.8a2 2 0 014 0V5\"></path></g></svg>",
  "moon":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M9.8 7.2A4.4 4.4 0 015 2.2a4.4 4.4 0 104.8 5z\"></path></svg>",
  "restart":  "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M2 6a4 4 0 111.2 2.8M3 11V8.5h2.5\"></path></svg>",
  "map":      "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M1.5 3l3-1.2L7.5 3l3-1.2v7.4l-3 1.2L4.5 9.2l-3 1.2V3z\"></path><path d=\"M4.5 1.8v7.4M7.5 3v7.4\"></path></g></svg>",
  "branch":   "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><circle cx=\"3\" cy=\"2.8\" r=\"1.4\"></circle><circle cx=\"9\" cy=\"2.8\" r=\"1.4\"></circle><circle cx=\"3\" cy=\"9.2\" r=\"1.4\"></circle><path d=\"M3 4.2v3.6M9 4.2c0 2.4-6 2.2-6 3.6\"></path></g></svg>",
  "extlink":  "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M5 2.5H2.5v7h7V7\"></path><path d=\"M6.5 1.5H10.5v4M10.5 1.5L5.8 6.2\"></path></g></svg>",
  "mic":      "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"4.4\" y=\"1.2\" width=\"3.2\" height=\"5.6\" rx=\"1.6\"></rect><path d=\"M2.6 5.6a3.4 3.4 0 006.8 0M6 9v1.8\"></path></g></svg>",
  "vol":      "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M2 4.5h1.8L6.5 2v8L3.8 7.5H2v-3z\" fill=\"currentColor\" stroke=\"none\"></path><path d=\"M8 4a2.8 2.8 0 010 4M9.5 2.8a4.8 4.8 0 010 6.4\"></path></g></svg>",
  "bt":       "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M3.5 3.5l5 5L6 11V1l2.5 2.5-5 5\"></path></svg>",
  "shield":   "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M6 1.2l4 1.5v3.2c0 2.6-1.7 4.3-4 5.1-2.3-.8-4-2.5-4-5.1V2.7l4-1.5z\"></path></svg>",
  "gear":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><circle cx=\"6\" cy=\"6\" r=\"1.7\"></circle><path d=\"M6 1.4v1.5M6 9.1v1.5M1.4 6h1.5M9.1 6h1.5M2.7 2.7l1.1 1.1M8.2 8.2l1.1 1.1M9.3 2.7L8.2 3.8M3.8 8.2L2.7 9.3\"></path></g></svg>",
  "music":    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M4.5 9.5V2.8l5-1v6.7\"></path><circle cx=\"3\" cy=\"9.5\" r=\"1.5\"></circle><circle cx=\"8\" cy=\"8.5\" r=\"1.5\"></circle></g></svg>",
  "chevR":    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M4.5 2.5L8 6l-3.5 3.5\"></path></svg>",
  "scissors": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><circle cx=\"3\" cy=\"3\" r=\"1.6\"></circle><circle cx=\"3\" cy=\"9\" r=\"1.6\"></circle><path d=\"M4.4 4L10.5 9.4M4.4 8L10.5 2.6\"></path></g></svg>",
  "area":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"1.5\" y=\"1.5\" width=\"9\" height=\"9\" rx=\"1\" stroke-dasharray=\"2.2 1.6\"></rect><rect x=\"4\" y=\"4\" width=\"4\" height=\"4\" rx=\"0.5\"></rect></g></svg>",
  "chip":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"2.5\" y=\"2.5\" width=\"7\" height=\"7\" rx=\"1.2\"></rect><path d=\"M4.5 1v1.5M7.5 1v1.5M4.5 9.5V11M7.5 9.5V11M1 4.5h1.5M1 7.5h1.5M9.5 4.5H11M9.5 7.5H11\"></path></g></svg>",
  "phones":   "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M2 8.5V6.5a4 4 0 018 0v2\"></path><rect x=\"1.3\" y=\"7.3\" width=\"2.3\" height=\"3.2\" rx=\"1\"></rect><rect x=\"8.4\" y=\"7.3\" width=\"2.3\" height=\"3.2\" rx=\"1\"></rect></g></svg>",
  "monitor":  "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><rect x=\"1.5\" y=\"2.3\" width=\"9\" height=\"6\" rx=\"1\"></rect><path d=\"M4.3 10.5h3.4M6 8.3v2.2\"></path></g></svg>",
  "pin":      "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M6 10.8S2.4 7.6 2.4 5a3.6 3.6 0 117.2 0c0 2.6-3.6 5.8-3.6 5.8z\"></path><circle cx=\"6\" cy=\"5\" r=\"1.2\"></circle></g></svg>",
  "cloud":    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M3.6 9.2h4.9a2.1 2.1 0 000-4.2 3 3 0 00-5.7.9 1.9 1.9 0 00.8 3.3z\"></path></svg>",
  "rain":     "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><path d=\"M3.6 7.2h4.9a2.1 2.1 0 000-4.2 3 3 0 00-5.7.9 1.9 1.9 0 00.8 3.3z\"></path><path d=\"M4 9l-.6 1.4M6.4 9l-.6 1.4M8.8 9l-.6 1.4\"></path></g></svg>",
  "partly":   "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><circle cx=\"7.8\" cy=\"4\" r=\"1.8\"></circle><path d=\"M7.8 1v.9M10.8 4h-.9M9.9 1.9l-.6.6\"></path><path d=\"M2.6 10h4.2a1.8 1.8 0 000-3.6 2.6 2.6 0 00-4.9.8A1.6 1.6 0 002.6 10z\" fill=\"var(--aero-panel-fill, #15171c)\"></path></g></svg>",
  "sun":      "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><g><circle cx=\"6\" cy=\"6\" r=\"2.1\" /><path d=\"M6 1v1.5M6 9.5V11M1 6h1.5M9.5 6H11M2.6 2.6l1 1M8.4 8.4l1 1M9.4 2.6l-1 1M3.6 8.4l-1 1\" /></g></svg>",
};

export type IconName = keyof typeof AERO_ICONS;

interface IconProps {
  name: IconName;
  size?: number;
  style?: CSSProperties;
}

/** Render a bundled Aeropeks icon at `size` px, colored via CSS `color`. */
export function Icon({ name, size = 12, style }: IconProps) {
  const raw = AERO_ICONS[name] ?? "";
  const html = raw
    .replace(/width="\d+"/, `width="${size}"`)
    .replace(/height="\d+"/, `height="${size}"`);
  return (
    <span
      style={{ display: 'inline-flex', width: size, height: size, flexShrink: 0, ...style }}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
