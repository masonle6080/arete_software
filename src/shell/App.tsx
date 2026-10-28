import { useEffect, useState } from "react";
import { areas } from "../areas";
import { libraryApi } from "../areas/library/api";
import { startSession, type SessionState } from "../areas/library/session";
import { WelcomeScreen } from "../areas/library/WelcomeScreen";

export function App() {
  const [session, setSession] = useState<SessionState | null>(null);
  const [areaId, setAreaId] = useState(areas[0].id);

  useEffect(() => {
    startSession(libraryApi).then(setSession);
  }, []);

  if (!session) return <main className="welcome">Opening library…</main>;

  if (session.status === "welcome") {
    return (
      <WelcomeScreen
        initialError={session.error}
        onOpened={(library) => setSession({ status: "open", library })}
      />
    );
  }

  const area = areas.find((a) => a.id === areaId) ?? areas[0];
  return (
    <div className="shell">
      <nav className="sidebar">
        {areas.map((a) => (
          <button key={a.id} className={a.id === area.id ? "active" : ""} onClick={() => setAreaId(a.id)}>
            {a.label}
          </button>
        ))}
        <button className="switch" onClick={() => setSession({ status: "welcome" })}>
          Switch library…
        </button>
      </nav>
      <main className="content">
        <area.Screen library={session.library} />
      </main>
    </div>
  );
}
