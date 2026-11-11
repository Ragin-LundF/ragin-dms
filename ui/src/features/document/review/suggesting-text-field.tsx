import { useRef, useState } from "react";
import ARROW_DROP_DOWN from "@material-design-icons/svg/round/arrow_drop_down.svg?raw";
import { IconButton, Menu, MenuItem, TextField } from "../../../shared/components";
import { shownOptions } from "./review-hints";

/**
 * A text field that offers `options` in a menu: typing shows the options containing the text, the arrow shows all of
 * them. Any other text may still be typed (Kotlin `SuggestingTextField`).
 */
export function SuggestingTextField({ value, label, options, onChange }: { value: string; label: string; options: string[]; onChange: (value: string) => void }) {
  const [open, setOpen] = useState(false);
  const [showAll, setShowAll] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const shown = shownOptions(options, value, showAll);
  return (
    <div ref={box}>
      <TextField
        label={label}
        value={value}
        labelBackground="var(--surface-high)"
        onChange={(typed) => {
          onChange(typed);
          setOpen(true);
          setShowAll(false);
        }}
        trailing={
          <IconButton
            icon={ARROW_DROP_DOWN}
            label="Show all"
            onMouseDown={(event) => event.stopPropagation()}
            onClick={() => {
              const all = !open || !showAll;
              setShowAll(all);
              setOpen(all);
            }}
          />
        }
      />
      {open && shown.length > 0 && box.current && (
        <Menu anchor={box.current.getBoundingClientRect()} onClose={() => setOpen(false)}>
          <div className="suggesting-options" style={{ width: box.current.getBoundingClientRect().width }}>
            {shown.map((option) => (
              <MenuItem
                key={option}
                label={option}
                onClick={() => {
                  onChange(option);
                  setOpen(false);
                }}
              />
            ))}
          </div>
        </Menu>
      )}
    </div>
  );
}
