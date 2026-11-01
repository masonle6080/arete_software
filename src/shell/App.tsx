import { useCallback, useEffect, useState } from "react";
import { libraryApi } from "../areas/library/api";
import { RECONNECT_INTERVAL_MS } from "../areas/library/reconnect";
import { ReconnectScreen } from "../areas/library/ReconnectScreen";
import { startSession, type LibraryInfo, type SessionState } from "../areas/library/session";
import { WelcomeScreen } from "../areas/library/WelcomeScreen";
import { onLibraryDisconnected } from "./invoke";
import { Workspace } from "./Workspace";

export function App() {
  const [session, setSession] = useState<SessionState | null>(null);

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

  // The title bar is hidden, so screens without the shell need their own strip to drag the window by.
  const dragStrip = <div className="drag-region window-drag" data-tauri-drag-region />;

  if (!session) return <main className="welcome">Opening library…</main>;

  if (session.status === "welcome") {
    return (
      <>
        {dragStrip}
        <WelcomeScreen
          initialError={session.error}
          onOpened={(library) => setSession({ status: "open", library })}
        />
      </>
    );
  }

  if (session.status === "disconnected") {
    return (
      <>
        {dragStrip}
        <ReconnectScreen
          library={session.library}
          onReconnected={onReconnected}
          onSwitch={() => setSession({ status: "welcome" })}
        />
      </>
    );
  }

  return <Workspace library={session.library} onSwitchLibrary={() => setSession({ status: "welcome" })} />;
}
