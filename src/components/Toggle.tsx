interface ToggleProps {
  label: string;
  checked: boolean;
  disabled?: boolean;
  onChange: (checked: boolean) => void;
}

/** Erişilebilir açma/kapama anahtarı (role="switch"). */
export function Toggle({ label, checked, disabled, onChange }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      className={`toggle${checked ? " is-on" : ""}`}
      onClick={() => {
        onChange(!checked);
      }}
    >
      <span className="toggle__knob" />
    </button>
  );
}
