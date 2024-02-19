import { zodResolver } from "@hookform/resolvers/zod";
import { AnimatePresence, motion } from "framer-motion";
import { AlertCircleIcon } from "lucide-react";
import { ControllerFieldState, useForm } from "react-hook-form";
import { z } from "zod";
import { schemas } from "~/api/schema";
import { AutosizeTextarea } from "~/components/autosize-textarea";
import { Button } from "~/components/ui/button";
import { Form, FormControl, FormField, FormItem } from "~/components/ui/form";
import { Task } from ".";
import { useUpdateTask } from "./use-update-task";
import { cn } from "~/lib/utils";
import { Separator } from "~/components/ui/separator";
import {
  ChangeEventHandler,
  ComponentPropsWithoutRef,
  forwardRef,
} from "react";

const updateTaskTitleSchema = schemas.UpdateTask.pick({ title: true });
type UpdateTaskTitle = z.infer<typeof updateTaskTitleSchema>;

type Props = { defaultTask: Task };
export const TaskTitleForm: React.FC<Props> = ({ defaultTask }) => {
  const form = useForm<UpdateTaskTitle>({
    defaultValues: { title: defaultTask.title },
    resolver: zodResolver(updateTaskTitleSchema),
  });
  const isDirty = form.formState.isDirty;

  const updateMutation = useUpdateTask();
  const handleSubmit = (data: UpdateTaskTitle) => {
    updateMutation.mutate(
      { ...defaultTask, ...data },
      {
        onSuccess: () => {
          form.reset({ title: data.title });
        },
      }
    );
  };

  const handleReset = () => {
    form.reset({ title: defaultTask.title });
  };

  return (
    <Form {...form}>
      <form className="relative" onSubmit={form.handleSubmit(handleSubmit)}>
        <FormField
          control={form.control}
          name="title"
          render={({ field, fieldState }) => {
            return (
              <TaskTitleFormField
                {...field}
                fieldState={fieldState}
                isDirty={isDirty}
                disabled={updateMutation.isPending}
                onReset={handleReset}
              />
            );
          }}
        />
      </form>
    </Form>
  );
};

type FieldProps = {
  fieldState: ControllerFieldState;
  isDirty: boolean;
  onReset: () => void;
} & ComponentPropsWithoutRef<"textarea">;
const TaskTitleFormField = forwardRef<HTMLTextAreaElement, FieldProps>(
  function TaskTitleFormField(
    { fieldState, isDirty, onReset, onChange, ...props },
    ref
  ) {
    const handleChangeTitle: ChangeEventHandler<HTMLTextAreaElement> = (e) => {
      e.target.value = e.target.value.replaceAll(/\n/g, "");
      onChange?.(e);
    };

    const handleResetTitle = () => {
      onReset();
    };

    return (
      <FormItem className="relative">
        <FormControl>
          <AutosizeTextarea
            ref={ref}
            className={cn(
              "text-2xl font-bold focus-visible:outline-none w-full rounded break-all",
              fieldState.error && "border border-destructive text-destructive"
            )}
            autoComplete="off"
            onChange={handleChangeTitle}
            {...props}
          />
        </FormControl>
        <AnimatePresence>
          {isDirty && (
            <motion.div
              className={cn(
                "absolute top-[105%] border border-border bg-card rounded-lg p-3 w-full",
                fieldState.error && "border-destructive"
              )}
              initial={{ opacity: 0, y: -5 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -5 }}
            >
              {fieldState.error && (
                <div>
                  <p className="text-xs text-destructive">
                    {fieldState.error.message}
                  </p>
                  <Separator className="my-2" />
                </div>
              )}
              <div className="flex justify-between items-center">
                <div className="flex gap-1 text-primary items-center grow">
                  <AlertCircleIcon size={15} />
                  <p className="text-xs">タイトルの変更が保存されていません</p>
                </div>
                <div className="flex gap-1 items-center">
                  <Button
                    size="sm"
                    variant="outline"
                    type="reset"
                    onClick={handleResetTitle}
                  >
                    リセットする
                  </Button>
                  <Button size="sm" disabled={props.disabled}>
                    保存する
                  </Button>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </FormItem>
    );
  }
);
