import { useState, useEffect } from "react";
import { WheelMode } from "../types/wheel";

export interface ModifiersState {
  shift: boolean;
  alt: boolean;
  ctrl: boolean;
  meta: boolean;
  activeMode: WheelMode | null;
}

export function useModifiers(): ModifiersState {
  const [modifiers, setModifiers] = useState<ModifiersState>({
    shift: false,
    alt: false,
    ctrl: false,
    meta: false,
    activeMode: null,
  });

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const shift = e.shiftKey;
      const alt = e.altKey;
      const ctrl = e.ctrlKey;
      const meta = e.metaKey;

      let activeMode: WheelMode | null = null;
      if (shift) {
        activeMode = "format";
      } else if (meta || alt) {
        activeMode = "tool";
      } else if (ctrl) {
        activeMode = "action";
      }

      setModifiers({ shift, alt, ctrl, meta, activeMode });
    }

    function handleKeyUp(e: KeyboardEvent) {
      const shift = e.shiftKey;
      const alt = e.altKey;
      const ctrl = e.ctrlKey;
      const meta = e.metaKey;

      let activeMode: WheelMode | null = null;
      if (shift) {
        activeMode = "format";
      } else if (meta || alt) {
        activeMode = "tool";
      } else if (ctrl) {
        activeMode = "action";
      }

      setModifiers({ shift, alt, ctrl, meta, activeMode });
    }

    function handleBlur() {
      setModifiers({
        shift: false,
        alt: false,
        ctrl: false,
        meta: false,
        activeMode: null,
      });
    }

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("blur", handleBlur);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("blur", handleBlur);
    };
  }, []);

  return modifiers;
}
