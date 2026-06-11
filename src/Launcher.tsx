import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Icon } from "./icons";

interface SearchResult {
  id: string;
  title: string;
  description: string;
  icon: string;
  action_type: string;
  action_value: string;
}

const Launcher = () => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => {
      if (query.trim()) {
        invoke<SearchResult[]>("search_query", { query })
          .then(setResults)
          .catch(console.error);
      } else {
        setResults([]);
      }
      setSelectedIndex(0);
    }, 150);
    return () => clearTimeout(timer);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      setSelectedIndex((prev) => (prev + 1) % results.length);
      e.preventDefault();
    } else if (e.key === "ArrowUp") {
      setSelectedIndex((prev) => (prev - 1 + results.length) % results.length);
      e.preventDefault();
    } else if (e.key === "Enter") {
      if (results[selectedIndex]) {
        handleLaunch(results[selectedIndex]);
      }
    } else if (e.key === "Escape") {
      invoke("toggle_launcher");
    }
  };

  const handleLaunch = (result: SearchResult) => {
    invoke("launch_result", { result })
      .then(() => {
        setQuery("");
        setResults([]);
      })
      .catch(console.error);
  };

  const getIcon = (iconName: string) => {
    switch (iconName) {
      case "Search":    return <Icon name="search"  size={14} />;
      case "AppWindow": return <Icon name="monitor" size={14} />;
      case "Settings":  return <Icon name="gear"    size={14} />;
      case "Globe":     return <Icon name="extlink" size={14} />;
      default:          return <Icon name="term"    size={14} />;
    }
  };

  return (
    <div className="launcher-container" data-tauri-drag-region onKeyDown={handleKeyDown}>
      <div className="launcher-input-wrapper">
        <span className="search-icon" style={{ display: 'flex' }}><Icon name="search" size={16} /></span>
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search for apps, files, or web..."
          className="launcher-input"
        />
      </div>
      
      {results.length > 0 && (
        <div className="launcher-results">
          {results.map((result, index) => (
            <div
              key={result.id}
              className={`launcher-result-item ${index === selectedIndex ? "selected" : ""}`}
              onClick={() => handleLaunch(result)}
              onMouseEnter={() => setSelectedIndex(index)}
            >
              <div className="result-icon">
                {getIcon(result.icon)}
              </div>
              <div className="result-text">
                <div className="result-title">{result.title}</div>
                <div className="result-description">{result.description}</div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default Launcher;
