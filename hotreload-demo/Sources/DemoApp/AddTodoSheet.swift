import SwiftUI

struct AddTodoSheet: View {
    @ObservedObject var store: TodoStore
    @Environment(\.dismiss) private var dismiss

    @State private var title = ""
    @State private var notes = ""
    @State private var priority: TodoPriority = .medium
    @State private var category = "Personal"

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Header
                    VStack(spacing: 8) {
                        Image(systemName: "plus.circle.fill")
                            .font(.system(size: 40))
                            .foregroundStyle(.purple)
                        Text("New Todo")
                            .font(.title2.bold())
                    }
                    .padding(.top, 4)

                    // Task fields
                    VStack(spacing: 12) {
                        HStack(spacing: 12) {
                            Image(systemName: "textformat")
                                .foregroundStyle(.secondary)
                                .frame(width: 16)
                            TextField("What needs to be done?", text: $title)
                        }
                        .padding()
                        .background(Color(.systemGray6))
                        .clipShape(RoundedRectangle(cornerRadius: 12))

                        HStack(alignment: .top, spacing: 12) {
                            Image(systemName: "note.text")
                                .foregroundStyle(.secondary)
                                .frame(width: 16)
                            TextField("Add notes...", text: $notes, axis: .vertical)
                                .lineLimit(3...6)
                        }
                        .padding()
                        .background(Color(.systemGray6))
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                    }

                    // Priority
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Priority")
                            .font(.caption.weight(.semibold))
                            .foregroundStyle(.secondary)
                            .padding(.leading, 2)

                        HStack(spacing: 10) {
                            ForEach(TodoPriority.allCases, id: \.self) { p in
                                Button {
                                    withAnimation(.spring(response: 0.3)) { priority = p }
                                } label: {
                                    HStack(spacing: 6) {
                                        Image(systemName: p.icon)
                                        Text(p.label)
                                    }
                                    .font(.subheadline.weight(.medium))
                                    .padding(.horizontal, 16)
                                    .padding(.vertical, 10)
                                    .frame(maxWidth: .infinity)
                                    .background(priority == p ? p.color.opacity(0.15) : Color(.systemGray6))
                                    .foregroundStyle(priority == p ? p.color : .secondary)
                                    .clipShape(RoundedRectangle(cornerRadius: 10))
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 10)
                                            .stroke(priority == p ? p.color : Color.clear, lineWidth: 1.5)
                                    )
                                }
                            }
                        }
                    }

                    // Category
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Category")
                            .font(.caption.weight(.semibold))
                            .foregroundStyle(.secondary)
                            .padding(.leading, 2)

                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 8) {
                                ForEach(TodoItem.categories, id: \.self) { cat in
                                    Button {
                                        withAnimation(.spring(response: 0.3)) { category = cat }
                                    } label: {
                                        Text(cat)
                                            .font(.subheadline.weight(.medium))
                                            .padding(.horizontal, 16)
                                            .padding(.vertical, 8)
                                            .background(category == cat ? Color.purple : Color(.systemGray6))
                                            .foregroundStyle(category == cat ? .white : .primary)
                                            .clipShape(Capsule())
                                    }
                                }
                            }
                        }
                    }

                    // Add button
                    Button {
                        store.add(title: title, notes: notes, priority: priority, category: category)
                        dismiss()
                    } label: {
                        Label("Add Todo", systemImage: "plus")
                            .font(.headline)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 14)
                            .background(title.isEmpty ? Color.gray.opacity(0.2) : Color.purple)
                            .foregroundStyle(title.isEmpty ? Color.gray : Color.white)
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                    }
                    .disabled(title.isEmpty)
                    .padding(.top, 4)
                }
                .padding(.horizontal, 20)
                .padding(.vertical, 8)
            }
            .background(Color(.systemGroupedBackground))
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
        }
    }
}
