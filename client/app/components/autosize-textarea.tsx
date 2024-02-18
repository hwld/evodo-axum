import { ChangeEventHandler, ComponentProps, forwardRef } from "react";
import { cn } from "~/lib/utils";

type Props = { className: string } & ComponentProps<"textarea">;
export const AutosizeTextarea = forwardRef<HTMLTextAreaElement, Props>(
  function AutosizeTextarea({ className, onChange, ...props }, ref) {
    const handleChange: ChangeEventHandler<HTMLTextAreaElement> = (e) => {
      onChange?.(e);
      e.target.style.height = "0px";
      e.target.style.height = `${e.target.scrollHeight}px`;
    };

    return (
      <textarea
        ref={ref}
        className={cn(className, "overflow-hidden resize-none block")}
        onChange={handleChange}
        {...props}
      ></textarea>
    );
  }
);
