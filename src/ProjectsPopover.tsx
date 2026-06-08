import { useMemo, useState } from "react";
import { ExternalLink, RefreshCw } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { Project, ProjectsSnapshot } from "./contracts";

const checkOrder = [
  "readme",
  "releases",
  "unreleased",
  "roadmap",
  "branding",
  "setup",
  "activity",
  "openActivity",
];

const timeAgo = (value: string | null) => {
  if (!value) return "never";
  const days = Math.floor((Date.now() - new Date(value).getTime()) / 86_400_000);
  if (days <= 0) return "today";
  if (days === 1) return "yesterday";
  if (days < 30) return `${days}d ago`;
  if (days < 365) return `${Math.floor(days / 30)}mo ago`;
  return `${Math.floor(days / 365)}y ago`;
};

const scoreClass = (score: number) =>
  score >= 80 ? "project-score-ok" : score >= 60 ? "project-score-warn" : "project-score-bad";

function ProjectRow({ project }: { project: Project }) {
  const open = (suffix = "") =>
    invoke("open_project_url", { url: `${project.url}${suffix}` }).catch(console.error);

  return (
    <div className="project-row">
      <button className="project-main" onClick={() => open()}>
        <strong className={scoreClass(project.healthScore)}>{project.healthScore}</strong>
        <span className="project-copy">
          <span className="project-name">
            {project.name}
            {project.isPrivate && <small>private</small>}
          </span>
          <span className="project-meta">{timeAgo(project.pushedAt)}</span>
        </span>
        <ExternalLink size={12} />
      </button>
      <div className="project-health-strip">
        {checkOrder.map((key) => {
          const item = project.checks[key];
          return item ? (
            <span
              key={key}
              className={`project-check project-check-${item.status}`}
              title={`${key}: ${item.detail}`}
            />
          ) : null;
        })}
      </div>
      <div className="project-links">
        <button onClick={() => open("/issues")}>issues</button>
        <button onClick={() => open("/pulls")}>pulls</button>
        <button onClick={() => open("/releases")}>releases</button>
      </div>
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
    const normalized = query.trim().toLowerCase();
    return normalized
      ? snapshot.projects.filter((project) =>
          project.fullName.toLowerCase().includes(normalized),
        )
      : snapshot.projects;
  }, [query, snapshot.projects]);

  return (
    <div className="projects-popover dropdown" onClick={(event) => event.stopPropagation()}>
      <div className="projects-header">
        <div>
          <strong>Projects</strong>
          <small>{snapshot.attentionCount} need attention</small>
        </div>
        <button onClick={onRefresh} disabled={refreshing} title="Refresh projects">
          <RefreshCw size={14} className={refreshing ? "spinning" : ""} />
        </button>
      </div>
      <input
        className="projects-search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Search repositories"
        autoFocus
      />
      <div className="projects-list">
        {projects.map((project) => (
          <ProjectRow project={project} key={project.fullName} />
        ))}
        {projects.length === 0 && <div className="projects-empty">No matching repositories</div>}
      </div>
    </div>
  );
}
