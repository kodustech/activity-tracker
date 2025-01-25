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
    <div className="space-y-6">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Categories
        </h2>

        <div className="space-y-4">
          {categories.map((category) => (
            <div
              key={category.id}
              className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700 rounded-lg"
            >
              <div className="flex items-center space-x-4">
                <div
                  className="w-4 h-4 rounded"
                  style={{ backgroundColor: category.color }}
                />
                <span className="text-gray-900 dark:text-white">
                  {category.name}
                </span>
              </div>
              <div className="flex items-center space-x-4">
                <label className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    checked={category.is_productive}
                    onChange={(e) =>
                      handleUpdateCategory({
                        ...category,
                        is_productive: e.target.checked,
                      })
                    }
                    className="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                  />
                  <span className="text-sm text-gray-600 dark:text-gray-300">
                    Productive
                  </span>
                </label>
                <button
                  onClick={() => handleDeleteCategory(category.id)}
                  className="text-red-500 hover:text-red-700"
                >
                  Delete
                </button>
              </div>
            </div>
          ))}

          <div className="mt-4 space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <input
                type="text"
                value={newCategory.name}
                onChange={(e) =>
                  setNewCategory({ ...newCategory, name: e.target.value })
                }
                placeholder="Category name"
                className="p-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800"
              />
              <input
                type="color"
                value={newCategory.color}
                onChange={(e) =>
                  setNewCategory({ ...newCategory, color: e.target.value })
                }
                className="p-1 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800"
              />
            </div>
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={newCategory.is_productive}
                onChange={(e) =>
                  setNewCategory({
                    ...newCategory,
                    is_productive: e.target.checked,
                  })
                }
                className="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
              />
              <span className="text-sm text-gray-600 dark:text-gray-300">
                Productive
              </span>
            </div>
            <button
              onClick={handleAddCategory}
              disabled={!newCategory.name}
              className="w-full p-2 bg-indigo-600 text-white rounded hover:bg-indigo-700 disabled:opacity-50"
            >
              Add Category
            </button>
          </div>
        </div>
      </div>

      {uncategorizedApps.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
            Uncategorized Applications
          </h2>
          <div className="space-y-2">
            {uncategorizedApps.map((app) => (
              <div
                key={app}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded"
              >
                <span className="text-gray-900 dark:text-white">{app}</span>
                <select
                  onChange={(e) => handleSetAppCategory(app, e.target.value)}
                  className="ml-4 p-1 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
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

      {/* Seção de Apps Categorizados */}
      {appCategories.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
            Categorized Applications
          </h2>
          <div className="space-y-2">
            {appCategories.map(({app_name, category_id}) => (
              <div
                key={app_name}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded"
              >
                <span className="text-gray-900 dark:text-white">{app_name}</span>
                <select
                  onChange={(e) => handleSetAppCategory(app_name, e.target.value)}
                  className="ml-4 p-1 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
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