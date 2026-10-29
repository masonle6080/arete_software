import { useEffect, useState, type FormEvent } from "react";
import type { Area } from "../../shell/area";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { currencyApi, type CurrencySettings } from "../costSheet/api";
import "./settings.css";

/**
 * Library settings: the home currency and the exchange rates the founder
 * enters by hand. Every Cost Sheet totals in the home currency with these
 * rates. There are no live rates.
 */
function SettingsScreen() {
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
      <h2>Currencies</h2>
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
        <button type="submit" disabled={home.trim() === "" || home === settings?.homeCurrency}>
          Save
        </button>
      </form>
      <p className="hint">Cost Sheets total each Quantity Tier in this currency, rounded to cents.</p>

      <h3>Exchange rates</h3>
      <p className="hint">One unit of each currency is worth this many units of {homeLabel}.</p>
      {settings && settings.rates.length === 0 ? (
        <p className="empty">No exchange rates yet.</p>
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
                  <button
                    onClick={() => {
                      setRateCurrency(r.currency);
                      setRate(r.rate);
                    }}
                  >
                    Edit
                  </button>
                  <button onClick={() => run(() => currencyApi.removeRate(r.currency))}>Remove</button>
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
        <button type="submit" disabled={rateCurrency.trim() === "" || rate.trim() === ""}>
          Save rate
        </button>
      </form>
    </section>
  );
}

export const settingsArea: Area = { id: "settings", label: "Settings", Screen: SettingsScreen };
