import { useMergedRef } from "@mantine/hooks";
import { ComponentProps, forwardRef, useLayoutEffect, useRef } from "react";
import { cn } from "~/lib/utils";

type Props = { className: string } & ComponentProps<"textarea">;
export const AutosizeTextarea = forwardRef<HTMLTextAreaElement, Props>(
  function AutosizeTextarea({ className, ...props }, ref) {
    const _textareaRef = useRef<HTMLTextAreaElement>(null);
    const textareaRef = useMergedRef(ref, _textareaRef);

    useLayoutEffect(() => {
      if (!_textareaRef.current) {
        return;
      }

      const textarea = _textareaRef.current;
      textarea.style.height = "0px";
      textarea.style.height = `${textarea.scrollHeight}px`;
    }, [props.value]);

    return (
      <textarea
        ref={textareaRef}
        className={cn(className, "overflow-hidden resize-none block")}
        {...props}
      ></textarea>
    );
  }
);
