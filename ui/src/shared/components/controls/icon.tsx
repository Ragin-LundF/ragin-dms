/** A material icon from the round set, tinted by the text colour: `import add from "@material-design-icons/svg/round/add.svg?raw"`. */
export function Icon({ svg, size, className, label }: { svg: string; size?: number; className?: string; label?: string }) {
  const style = size === undefined ? undefined : { width: size, height: size };
  return (
    <span
      className={className ? `icon ${className}` : "icon"}
      style={style}
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
      // The markup comes from the bundled icon package, never from user data.
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}
