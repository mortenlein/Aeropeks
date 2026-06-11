import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Icon } from "./icons";
import type { Project, ProjectsSnapshot } from "./contracts";
import { Panel, Mono } from "./atoms";
import { HUE, T } from "./tokens";

const timeAgo = (value: string | null) => {
  if (!value) return "never";
  const days = Math.floor((Date.now() - new Date(value).getTime()) / 86_400_000);
  if (days <= 0) return "today";
  if (days === 1) return "yesterday";
  if (days < 30) return `${days}d ago`;
  if (days < 365) return `${Math.floor(days / 30)}mo ago`;
  return `${Math.floor(days / 365)}y ago`;
};

function ProjectRow({ project, index }: { project: Project; index: number }) {
  const open = (suffix = "") =>
    invoke("open_project_url", { url: `${project.url}${suffix}` }).catch(console.error);

  const scoreHue = project.healthScore >= 40 ? HUE.red : HUE.amber;

  return (
    <div
      style={{ display: "flex", alignItems: "center", gap: 10, padding: "10px 2px", borderTop: index > 0 ? `1px solid ${T.divider}` : "none", cursor: "pointer" }}
      onClick={() => open()}
    >
      <Mono size={13} w={700} color={scoreHue} style={{ width: 26, textAlign: "right", flexShrink: 0 }}>
        {project.healthScore}
      </Mono>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
          <span style={{ fontSize: 12.5, fontWeight: 600, color: T.t1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {project.name}
          </span>
          {project.isPrivate && (
            <span style={{ fontFamily: "var(--font-mono)", fontSize: 7.5, padding: "1.5px 5px", borderRadius: T.pillR, border: `1px solid ${T.divider}`, color: T.t3, letterSpacing: "0.08em", textTransform: "uppercase", flexShrink: 0 }}>
              private
            </span>
          )}
        </div>
        <div style={{ display: "flex", gap: 8, marginTop: 4 }}>
          <Mono size={9.5} color={T.t3} w={500}>{project.openIssuesCount} issues</Mono>
          <Mono size={9.5} color={T.t3} w={500}>{project.openPrsCount} pulls</Mono>
          <Mono size={9.5} color={T.t3} w={500}>{project.releasesCount} releases</Mono>
          <Mono size={9.5} color={T.t3} w={500} style={{ marginLeft: "auto" }}>{timeAgo(project.pushedAt)}</Mono>
        </div>
      </div>
      <span style={{ color: T.t3, display: "flex", cursor: "pointer", flexShrink: 0 }} onClick={(e) => { e.stopPropagation(); open(); }}>
        <Icon name="extlink" size={11} />
      </span>
    </div>
  );
}

export function ProjectsPopover({
  snapshot,
  refreshing,
  onRefresh,
}: {
  snapshot: ProjectsSnapshot;
  refreshing: boolean;
  onRefresh: () => void;
}) {
  const [query, setQuery] = useState("");
  const projects = useMemo(() => {
    const n = query.trim().toLowerCase();
    return n
      ? snapshot.projects.filter((p) => p.fullName.toLowerCase().includes(n))
      : snapshot.projects;
  }, [query, snapshot.projects]);

  return (
    <Panel
      w={360}
      title="Projects"
      hue="var(--accent)"
      style={{ left: 0 }}
      actions={
        <span
          onClick={onRefresh}
          style={{ cursor: refreshing ? "not-allowed" : "pointer", color: T.t3, display: "flex", opacity: refreshing ? 0.4 : 1 }}
        >
          <Icon name="refresh" size={11} style={{ animation: refreshing ? "spin 1s linear infinite" : "none" }} />
        </span>
      }
    >
      {/* Search bar with attention count inline */}
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12, padding: "7px 10px", background: T.inputBg, border: T.inputBorder, borderRadius: T.ctlR }}>
        <Icon name="search" size={11} style={{ color: T.t3, flexShrink: 0 }} />
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search repositories"
          autoFocus
          style={{
            flex: 1,
            background: "none",
            border: "none",
            fontFamily: "var(--font-ui)",
            fontSize: 12,
            color: T.t1,
            outline: "none",
          }}
        />
        {snapshot.attentionCount > 0 && (
          <Mono size={9.5} color={T.t3}>{snapshot.attentionCount} flagged</Mono>
        )}
      </div>

      {/* Rows — plain dividers, no card backgrounds */}
      <div style={{ maxHeight: 400, overflowY: "auto" }}>
        {projects.map((project, i) => (
          <ProjectRow project={project} key={project.fullName} index={i} />
        ))}
        {projects.length === 0 && (
          <div style={{ textAlign: "center", padding: "20px 0", fontSize: 12, color: T.t3 }}>
            No matching repositories
          </div>
        )}
      </div>
    </Panel>
  );
}
