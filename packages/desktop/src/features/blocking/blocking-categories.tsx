// features/blocking/blocking-categories.tsx - Category-based blocking management

import { useState } from "react";
import { Grid3x3, Plus, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { toast } from "sonner";
import {
  useBlockingCategories,
  useCreateCategory,
  useUpdateCategory,
  useToggleCategory,
} from "@/hooks/use-blocking-advanced";

export function BlockingCategories() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [editingCategory, setEditingCategory] = useState<number | null>(null);
  const [newCategoryName, setNewCategoryName] = useState("");
  const [newCategoryDescription, setNewCategoryDescription] = useState("");
  const [newCategoryItems, setNewCategoryItems] = useState<string[]>([]);
  const [itemInput, setItemInput] = useState("");

  const { data: categories, isLoading } = useBlockingCategories();
  const createCategory = useCreateCategory();
  const updateCategory = useUpdateCategory();
  const toggleCategory = useToggleCategory();

  // Suppress unused variable warnings - these will be used when edit functionality is implemented
  void editingCategory;
  void updateCategory;

  const handleCreateCategory = async () => {
    if (!newCategoryName.trim()) {
      toast.error("Category name is required");
      return;
    }

    if (newCategoryItems.length === 0) {
      toast.error("Add at least one site or app");
      return;
    }

    try {
      await createCategory.mutateAsync({
        name: newCategoryName,
        description: newCategoryDescription || undefined,
        items: newCategoryItems,
      });
      toast.success("Category created successfully");
      resetForm();
      setIsCreateDialogOpen(false);
    } catch (error) {
      toast.error("Failed to create category");
      console.error(error);
    }
  };

  const handleToggle = async (id: number, enabled: boolean) => {
    try {
      await toggleCategory.mutateAsync({ id, enabled });
      toast.success(`Category ${enabled ? "enabled" : "disabled"}`);
    } catch (error) {
      toast.error("Failed to toggle category");
      console.error(error);
    }
  };

  const handleAddItem = () => {
    const trimmedItem = itemInput.trim().toLowerCase();
    if (!trimmedItem) return;

    if (newCategoryItems.includes(trimmedItem)) {
      toast.error("Item already added");
      return;
    }

    setNewCategoryItems([...newCategoryItems, trimmedItem]);
    setItemInput("");
  };

  const handleRemoveItem = (item: string) => {
    setNewCategoryItems(newCategoryItems.filter((i) => i !== item));
  };

  const resetForm = () => {
    setNewCategoryName("");
    setNewCategoryDescription("");
    setNewCategoryItems([]);
    setItemInput("");
    setEditingCategory(null);
  };

  // Predefined categories (read-only)
  const predefinedCategories = Array.isArray(categories)
    ? categories.filter((cat) =>
        ["Social Media", "News", "Gaming", "Video", "Shopping"].includes(cat.name)
      )
    : [];

  // Custom categories
  const customCategories = Array.isArray(categories)
    ? categories.filter(
        (cat) => !["Social Media", "News", "Gaming", "Video", "Shopping"].includes(cat.name)
      )
    : [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Blocking Categories</h3>
          <p className="text-sm text-muted-foreground">
            Block entire categories of sites and apps at once
          </p>
        </div>
        <Button onClick={() => setIsCreateDialogOpen(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Create Category
        </Button>
      </div>

      {isLoading ? (
        <div className="text-sm text-muted-foreground">Loading categories...</div>
      ) : (
        <div className="space-y-6">
          {/* Predefined Categories */}
          {predefinedCategories && predefinedCategories.length > 0 && (
            <div className="space-y-3">
              <h4 className="text-sm font-medium">Predefined Categories</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {predefinedCategories.map((category) => (
                  <Card key={category.id} className={!category.enabled ? "opacity-60" : ""}>
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between">
                        <div className="flex items-center gap-2">
                          <Grid3x3 className="h-4 w-4 text-muted-foreground" />
                          <CardTitle className="text-base">{category.name}</CardTitle>
                        </div>
                        <Switch
                          checked={category.enabled}
                          onCheckedChange={(checked) => handleToggle(category.id, checked)}
                        />
                      </div>
                      {category.description && (
                        <CardDescription className="text-xs">
                          {category.description}
                        </CardDescription>
                      )}
                    </CardHeader>
                    <CardContent>
                      <div className="flex flex-wrap gap-1">
                        {category.items.slice(0, 5).map((item) => (
                          <Badge key={item} variant="secondary" className="text-xs">
                            {item}
                          </Badge>
                        ))}
                        {category.items.length > 5 && (
                          <Badge variant="outline" className="text-xs">
                            +{category.items.length - 5} more
                          </Badge>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          )}

          {/* Custom Categories */}
          {customCategories && customCategories.length > 0 && (
            <div className="space-y-3">
              <h4 className="text-sm font-medium">Custom Categories</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {customCategories.map((category) => (
                  <Card key={category.id} className={!category.enabled ? "opacity-60" : ""}>
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between">
                        <div className="flex items-center gap-2">
                          <Grid3x3 className="h-4 w-4 text-muted-foreground" />
                          <CardTitle className="text-base">{category.name}</CardTitle>
                        </div>
                        <div className="flex items-center gap-2">
                          <Switch
                            checked={category.enabled}
                            onCheckedChange={(checked) => handleToggle(category.id, checked)}
                          />
                        </div>
                      </div>
                      {category.description && (
                        <CardDescription className="text-xs">
                          {category.description}
                        </CardDescription>
                      )}
                    </CardHeader>
                    <CardContent>
                      <div className="flex flex-wrap gap-1">
                        {category.items.slice(0, 5).map((item) => (
                          <Badge key={item} variant="secondary" className="text-xs">
                            {item}
                          </Badge>
                        ))}
                        {category.items.length > 5 && (
                          <Badge variant="outline" className="text-xs">
                            +{category.items.length - 5} more
                          </Badge>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Create Category Dialog */}
      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Create Custom Category</DialogTitle>
            <DialogDescription>
              Group related sites and apps into a category you can enable/disable together
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="category-name">Category Name</Label>
              <Input
                id="category-name"
                placeholder="e.g., Work Tools"
                value={newCategoryName}
                onChange={(e) => setNewCategoryName(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="category-description">Description (optional)</Label>
              <Input
                id="category-description"
                placeholder="Brief description"
                value={newCategoryDescription}
                onChange={(e) => setNewCategoryDescription(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="category-items">Sites & Apps</Label>
              <div className="flex gap-2">
                <Input
                  id="category-items"
                  placeholder="e.g., slack.com or Slack"
                  value={itemInput}
                  onChange={(e) => setItemInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      handleAddItem();
                    }
                  }}
                />
                <Button type="button" onClick={handleAddItem}>
                  Add
                </Button>
              </div>

              {newCategoryItems.length > 0 && (
                <div className="flex flex-wrap gap-2 mt-2">
                  {newCategoryItems.map((item) => (
                    <Badge key={item} variant="secondary" className="flex items-center gap-1 pr-1">
                      {item}
                      <button
                        onClick={() => handleRemoveItem(item)}
                        className="ml-1 rounded-full p-0.5 hover:bg-muted"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => { resetForm(); setIsCreateDialogOpen(false); }}>
              Cancel
            </Button>
            <Button onClick={handleCreateCategory} disabled={createCategory.isPending}>
              Create Category
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
