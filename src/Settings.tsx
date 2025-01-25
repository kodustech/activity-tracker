import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface Category {
  id: string;
  name: string;
  color: string;
  is_productive: boolean;
}

interface AppCategory {
  app_name: string;
  category_id: string;
}

export function Settings() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [appCategories, setAppCategories] = useState<AppCategory[]>([]);
  const [uncategorizedApps, setUncategorizedApps] = useState<string[]>([]);
  const [newCategory, setNewCategory] = useState({
    name: '',
    color: '#6366f1',
    is_productive: false,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [categoriesResult, appCategoriesResult, uncategorizedResult] = await Promise.all([
        invoke<Category[]>('get_categories'),
        invoke<[string, string][]>('get_app_categories'),
        invoke<string[]>('get_uncategorized_apps'),
      ]);

      setCategories(categoriesResult);
      setAppCategories(
        appCategoriesResult.map(([app_name, category_id]) => ({
          app_name,
          category_id,
        }))
      );
      setUncategorizedApps(uncategorizedResult);
    } catch (err) {
      console.error('Error loading data:', err);
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handleAddCategory = async () => {
    try {
      const category = await invoke<Category>('add_category', {
        name: newCategory.name,
        color: newCategory.color,
        is_productive: newCategory.is_productive,
      });

      setCategories([...categories, category]);
      setNewCategory({ name: '', color: '#6366f1', is_productive: false });
    } catch (err) {
      console.error('Error adding category:', err);
      setError(err instanceof Error ? err.message : 'Failed to add category');
    }
  };

  const handleUpdateCategory = async (category: Category) => {
    try {
      await invoke('update_category', {
        id: category.id,
        name: category.name,
        color: category.color,
        is_productive: category.is_productive,
      });

      setCategories(
        categories.map((c) => (c.id === category.id ? category : c))
      );
    } catch (err) {
      console.error('Error updating category:', err);
      setError(err instanceof Error ? err.message : 'Failed to update category');
    }
  };

  const handleDeleteCategory = async (id: string) => {
    try {
      await invoke('delete_category', { id });
      setCategories(categories.filter((c) => c.id !== id));
      await loadData(); // Recarrega os dados para atualizar a lista de apps não categorizados
    } catch (err) {
      console.error('Error deleting category:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete category');
    }
  };

  const handleSetAppCategory = async (appName: string, categoryId: string) => {
    try {
      console.log('Setting category', { appName, categoryId });
      const result = await invoke('set_app_category', {
        app_name: appName,
        category_id: categoryId,
      });
      console.log('Category set successfully', result);
      await loadData();
    } catch (err) {
      console.error('Detailed error setting app category:', err);
      setError(err instanceof Error ? err.message : 'Failed to set app category');
    }
  };

  if (loading) {
    return <div className="text-center py-4">Loading...</div>;
  }

  if (error) {
    return <div className="text-red-500 text-center py-4">{error}</div>;
  }

  return (
    <div className="space-y-8">
      <div className="card">
        <h2 className="text-lg font-medium mb-6">Categories</h2>

        <div className="space-y-4">
          {categories.map((category) => (
            <div
              key={category.id}
              className="flex items-center justify-between p-3 bg-[var(--surface-hover)] rounded-md"
            >
              <div className="flex items-center space-x-3">
                <div
                  className="w-3 h-3 rounded"
                  style={{ backgroundColor: category.color }}
                />
                <span>{category.name}</span>
              </div>
              <div className="flex items-center space-x-2">
                <button
                  onClick={() => handleDeleteCategory(category.id)}
                  className="btn-secondary text-sm"
                >
                  Delete
                </button>
              </div>
            </div>
          ))}

          <div className="pt-4 border-t border-[var(--border)]">
            <div className="flex gap-4 mb-4">
              <input
                type="text"
                value={newCategory.name}
                onChange={(e) =>
                  setNewCategory({ ...newCategory, name: e.target.value })
                }
                placeholder="Category name"
                className="flex-1"
              />
              <input
                type="color"
                value={newCategory.color}
                onChange={(e) =>
                  setNewCategory({ ...newCategory, color: e.target.value })
                }
                className="w-12 h-10 p-1 bg-[var(--surface)] rounded border border-[var(--border)]"
              />
            </div>
            <div className="flex items-center justify-between">
              <label className="flex items-center space-x-2 text-[var(--text-secondary)]">
                <input
                  type="checkbox"
                  checked={newCategory.is_productive}
                  onChange={(e) =>
                    setNewCategory({
                      ...newCategory,
                      is_productive: e.target.checked,
                    })
                  }
                  className="rounded border-[var(--border)]"
                />
                <span>Is Productive</span>
              </label>
              <button
                onClick={handleAddCategory}
                disabled={!newCategory.name}
                className="btn-primary"
              >
                Add Category
              </button>
            </div>
          </div>
        </div>
      </div>

      {uncategorizedApps.length > 0 && (
        <div className="card">
          <h2 className="text-lg font-medium mb-6">Uncategorized Applications</h2>
          <div className="space-y-3">
            {uncategorizedApps.map((app) => (
              <div
                key={app}
                className="flex items-center justify-between p-3 bg-[var(--surface-hover)] rounded-md"
              >
                <span>{app}</span>
                <select
                  onChange={(e) => handleSetAppCategory(app, e.target.value)}
                  className="w-48"
                  defaultValue=""
                >
                  <option value="" disabled>
                    Select a category
                  </option>
                  {categories.map((category) => (
                    <option key={category.id} value={category.id}>
                      {category.name}
                    </option>
                  ))}
                </select>
              </div>
            ))}
          </div>
        </div>
      )}

      {appCategories.length > 0 && (
        <div className="card">
          <h2 className="text-lg font-medium mb-6">Categorized Applications</h2>
          <div className="space-y-3">
            {appCategories.map(({app_name, category_id}) => (
              <div
                key={app_name}
                className="flex items-center justify-between p-3 bg-[var(--surface-hover)] rounded-md"
              >
                <span>{app_name}</span>
                <select
                  onChange={(e) => handleSetAppCategory(app_name, e.target.value)}
                  className="w-48"
                  value={category_id}
                >
                  {categories.map((category) => (
                    <option key={category.id} value={category.id}>
                      {category.name}
                    </option>
                  ))}
                </select>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
} 