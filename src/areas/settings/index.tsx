import { useEffect, useState, type FormEvent } from "react";
import type { Area, AreaProps } from "../../shell/area";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { currencyApi, type CurrencySettings } from "../costSheet/api";
import "./settings.css";
import { Button, EmptyText, SectionHeader } from "../../ui";

/**
 * Library settings: the Library folder, and the home currency and the
 * exchange rates the founder enters by hand. Every Cost Sheet totals in the
 * home currency with these rates. There are no live rates.
 */
function SettingsScreen({ library }: AreaProps) {
  const [settings, setSettings] = useState<CurrencySettings>();
  const [home, setHome] = useState("");
  const [rateCurrency, setRateCurrency] = useState("");
  const [rate, setRate] = useState("");
  const [error, setError] = useState<CoreError>();

  useEffect(() => {
    currencyApi.settings().then(
      (s) => {
        setSettings(s);
        setHome(s.homeCurrency ?? "");
      },
      (raw) => setError(toCoreError(raw)),
    );
  }, []);

  async function run(action: () => Promise<CurrencySettings>): Promise<boolean> {
    setError(undefined);
    try {
      const next = await action();
      setSettings(next);
      setHome(next.homeCurrency ?? "");
      return true;
    } catch (raw) {
      setError(toCoreError(raw));
      return false;
    }
  }

  function saveHome(e: FormEvent) {
    e.preventDefault();
    run(() => currencyApi.setHomeCurrency(home));
  }

  async function saveRate(e: FormEvent) {
    e.preventDefault();
    if (await run(() => currencyApi.setRate(rateCurrency, rate))) {
      setRateCurrency("");
      setRate("");
    }
  }

  const homeLabel = settings?.homeCurrency ?? "home currency";

  return (
    <section className="settings">
      <h1>Settings</h1>
      {error && <p className="error">{error.message}</p>}
      <SectionHeader title="Library" />
      <dl className="facts">
        <dt>Folder</dt>
        <dd>{library.path}</dd>
        <dt>Schema version</dt>
        <dd>{library.schemaVersion}</dd>
      </dl>
      <SectionHeader title="Currencies" />
      <form className="settings-row" onSubmit={saveHome}>
        <label>
          Home currency
          <input
            className="currency"
            maxLength={3}
            placeholder="USD"
            value={home}
            onChange={(e) => setHome(e.target.value.toUpperCase())}
          />
        </label>
        <Button type="submit" disabled={home.trim() === "" || home === settings?.homeCurrency}>
          Save
        </Button>
      </form>
      <p className="hint">Cost Sheets total each Quantity Tier in this currency, rounded to cents.</p>

      <SectionHeader level={3} title="Exchange rates" />
      <p className="hint">One unit of each currency is worth this many units of {homeLabel}.</p>
      {settings && settings.rates.length === 0 ? (
        <EmptyText>No exchange rates yet.</EmptyText>
      ) : (
        <table className="rates">
          <tbody>
            {settings?.rates.map((r) => (
              <tr key={r.currency} className={r.currency === settings.homeCurrency ? "unused" : undefined}>
                <td>1 {r.currency}</td>
                <td>=</td>
                <td className="rate">
                  {r.rate} {homeLabel}
                </td>
                <td>
                  <Button
                    onClick={() => {
                      setRateCurrency(r.currency);
                      setRate(r.rate);
                    }}
                  >
                    Edit
                  </Button>
                  <Button onClick={() => run(() => currencyApi.removeRate(r.currency))}>Remove</Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <form className="settings-row" onSubmit={saveRate}>
        <label>
          Currency
          <input
            className="currency"
            maxLength={3}
            placeholder="EUR"
            value={rateCurrency}
            onChange={(e) => setRateCurrency(e.target.value.toUpperCase())}
          />
        </label>
        <label>
          Rate
          <input inputMode="decimal" placeholder="1.08" value={rate} onChange={(e) => setRate(e.target.value)} />
        </label>
        <Button type="submit" disabled={rateCurrency.trim() === "" || rate.trim() === ""}>
          Save rate
        </Button>
      </form>
    </section>
  );
}

export const settingsArea: Area = { id: "settings", label: "Settings", Screen: SettingsScreen };
