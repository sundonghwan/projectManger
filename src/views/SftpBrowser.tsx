import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { ServerConnection, SftpEntry } from "../domain/types";
import { SftpList } from "./SftpList";

export interface SftpBrowserProps {
  server: ServerConnection;
  onClose: () => void;
}

export function SftpBrowser({ server, onClose }: SftpBrowserProps) {
  const [path, setPath] = useState(".");
  const [entries, setEntries] = useState<SftpEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(
    async (p: string) => {
      try {
        setEntries(await api.sftp.list(server.id, p));
        setError(null);
      } catch (e) {
        setError(String(e));
        setEntries([]);
      }
    },
    [server.id],
  );

  useEffect(() => {
    void load(path);
  }, [path, load]);

  const onOpen = (e: SftpEntry) => setPath(path === "." ? e.name : `${path}/${e.name}`);
  const onUp = () => {
    if (path === "." || !path.includes("/")) setPath(".");
    else setPath(path.split("/").slice(0, -1).join("/") || ".");
  };

  return (
    <SftpList path={path} entries={entries} error={error} onUp={onUp} onOpen={onOpen} onClose={onClose} />
  );
}
