import { useCallback, useEffect, useState } from "react";
import { areas } from "../areas";
import { libraryApi } from "../areas/library/api";
import { RECONNECT_INTERVAL_MS } from "../areas/library/reconnect";
import { ReconnectScreen } from "../areas/library/ReconnectScreen";
import { startSession, type LibraryInfo, type SessionState } from "../areas/library/session";
import { WelcomeScreen } from "../areas/library/WelcomeScreen";
import { onLibraryDisconnected } from "./invoke";

export function App() {
  const [session, setSession] = useState<SessionState | null>(null);
  const [areaId, setAreaId] = useState(areas[0].id);

  useEffect(() => {
    startSession(libraryApi).then(setSession);
  }, []);

  // While a library is open, watch for its folder going away: either a
  // command fails with LibraryDisconnected, or the periodic check sees it.
  const openLibrary = session?.status === "open" ? session.library : null;
  useEffect(() => {
    if (!openLibrary) return;
    const disconnect = () => setSession({ status: "disconnected", library: openLibrary });
    const unsubscribe = onLibraryDisconnected(disconnect);
    const timer = setInterval(() => {
      libraryApi.libraryConnected().then((ok) => ok || disconnect(), () => {});
    }, RECONNECT_INTERVAL_MS);
    return () => {
      unsubscribe();
      clearInterval(timer);
    };
  }, [openLibrary]);

  const onReconnected = useCallback((library: LibraryInfo) => setSession({ status: "open", library }), []);

  if (!session) return <main className="welcome">Opening library…</main>;

  if (session.status === "welcome") {
    return (
      <WelcomeScreen
        initialError={session.error}
        onOpened={(library) => setSession({ status: "open", library })}
      />
    );
  }

  if (session.status === "disconnected") {
    return (
      <ReconnectScreen
        library={session.library}
        onReconnected={onReconnected}
        onSwitch={() => setSession({ status: "welcome" })}
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
