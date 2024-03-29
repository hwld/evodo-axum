import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { schemas } from "~/api/schema";
import { Textarea } from "~/components/ui/textarea";
import { Task } from ".";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormMessage,
} from "~/components/ui/form";
import { cn } from "~/lib/utils";
import { Button } from "~/components/ui/button";
import { AlertCircleIcon } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { useUpdateTask } from "./use-update-task";

const updateTaskDescriptionSchema = schemas.UpdateTask.pick({
  description: true,
});
type UpdateTaskDescription = z.infer<typeof updateTaskDescriptionSchema>;

type Props = { defaultTask: Task };
export const TaskDescriptionForm: React.FC<Props> = ({ defaultTask }) => {
  const form = useForm<UpdateTaskDescription>({
    defaultValues: {
      description: defaultTask.description,
    },
    resolver: zodResolver(updateTaskDescriptionSchema),
  });

  const isDirty = form.formState.isDirty;

  const updateMutation = useUpdateTask();
  const handleSubmit = (data: UpdateTaskDescription) => {
    updateMutation.mutate(
      { ...defaultTask, ...data },
      {
        onSuccess: (data) => {
          form.reset({ description: data.description });
        },
      }
    );
  };

  const handleReset = () => {
    form.reset({ description: defaultTask.description });
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)}>
        <FormField
          control={form.control}
          name="description"
          render={({ field, fieldState }) => {
            return (
              <FormItem>
                <FormControl>
                  <Textarea
                    placeholder="タスクの説明を入力できます..."
                    autoComplete="off"
                    className={cn(
                      "h-[500px] resize-none",
                      fieldState.error &&
                        "border-destructive focus-visible:ring-destructive"
                    )}
                    {...field}
                    disabled={updateMutation.isPending}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            );
          }}
        />
        <AnimatePresence>
          {isDirty && (
            <motion.div
              className="flex gap- justify-between items-center"
              initial={{ opacity: 0, y: -5 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -5 }}
            >
              <div className="text-xs flex gap-1">
                <AlertCircleIcon size={15} />
                変更が保存されていません。
              </div>
              <div className="flex gap-2">
                <Button
                  type="reset"
                  size="sm"
                  variant="outline"
                  onClick={handleReset}
                >
                  リセットする
                </Button>
                <Button
                  size="sm"
                  disabled={updateMutation.isPending || !isDirty}
                >
                  保存する
                </Button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </form>
    </Form>
  );
};
