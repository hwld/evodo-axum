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
import { useMutation } from "@tanstack/react-query";
import { api } from "~/api/index.client";
import { toast } from "sonner";
import { useRevalidator } from "@remix-run/react";
import { Button } from "~/components/ui/button";
import { AlertCircleIcon } from "lucide-react";

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

  const revalidator = useRevalidator();
  const updateMutation = useMutation({
    mutationFn: (data: UpdateTaskDescription) => {
      return api.put(
        "/tasks/:id",
        { ...defaultTask, description: data.description },
        { params: { id: defaultTask.id } }
      );
    },
    onError: (e) => {
      console.error(e);
      toast.error("タスクを更新できませんでした。");
    },
    onSuccess: (data) => {
      revalidator.revalidate();
      form.reset({ description: data.description });
    },
  });

  const handleSubmit = (data: UpdateTaskDescription) => {
    updateMutation.mutate(data);
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
        {isDirty && (
          <div className="flex gap- justify-between items-center">
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
              <Button size="sm" disabled={updateMutation.isPending || !isDirty}>
                保存する
              </Button>
            </div>
          </div>
        )}
      </form>
    </Form>
  );
};
