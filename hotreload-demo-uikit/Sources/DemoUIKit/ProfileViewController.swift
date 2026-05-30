import UIKit

class ProfileViewController: UIViewController {

    override func viewDidLoad() {
        super.viewDidLoad()
        title = "Profile"
        setupUI()
    }

    private func setupUI() {
        view.backgroundColor = .systemCyan

        let stack = UIStackView()
        stack.axis = .vertical
        stack.spacing = 20
        stack.alignment = .center
        stack.translatesAutoresizingMaskIntoConstraints = false

        // Avatar
        let avatar = UIImageView(image: UIImage(systemName: "person.circle.fill"))
        avatar.tintColor = .systemBlue
        avatar.contentMode = .scaleAspectFit
        avatar.translatesAutoresizingMaskIntoConstraints = false
        avatar.heightAnchor.constraint(equalToConstant: 100).isActive = true
        avatar.widthAnchor.constraint(equalToConstant: 100).isActive = true

        // Name
        let nameLabel = UILabel()
        nameLabel.text = "Ari Munandar"
        nameLabel.font = .systemFont(ofSize: 24, weight: .bold)

        // Role
        let roleLabel = UILabel()
        roleLabel.text = "iOS Developer"
        roleLabel.font = .systemFont(ofSize: 16)
        roleLabel.textColor = .secondaryLabel

        // Stats row
        let statsStack = UIStackView()
        statsStack.axis = .horizontal
        statsStack.spacing = 32
        statsStack.alignment = .center

        statsStack.addArrangedSubview(makeStatView(value: "42", label: "Projects"))
        statsStack.addArrangedSubview(makeStatView(value: "128", label: "Commits"))
        statsStack.addArrangedSubview(makeStatView(value: "3.5k", label: "Stars"))

        // Bio card
        let bioCard = makeBioCard(
            text: "Building tools that make iOS development faster. Hot reload enthusiast."
        )

        stack.addArrangedSubview(avatar)
        stack.addArrangedSubview(nameLabel)
        stack.addArrangedSubview(roleLabel)
        stack.addArrangedSubview(statsStack)
        stack.addArrangedSubview(bioCard)

        view.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            stack.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 40),
            stack.leadingAnchor.constraint(greaterThanOrEqualTo: view.leadingAnchor, constant: 20),
            stack.trailingAnchor.constraint(lessThanOrEqualTo: view.trailingAnchor, constant: -20),
        ])
    }

    private func makeStatView(value: String, label: String) -> UIView {
        let stack = UIStackView()
        stack.axis = .vertical
        stack.alignment = .center
        stack.spacing = 4

        let valueLabel = UILabel()
        valueLabel.text = value
        valueLabel.font = .systemFont(ofSize: 20, weight: .bold)
        valueLabel.textColor = .systemBlue

        let titleLabel = UILabel()
        titleLabel.text = label
        titleLabel.font = .systemFont(ofSize: 12)
        titleLabel.textColor = .tertiaryLabel

        stack.addArrangedSubview(valueLabel)
        stack.addArrangedSubview(titleLabel)
        return stack
    }

    private func makeBioCard(text: String) -> UIView {
        let card = UIView()
        card.backgroundColor = .secondarySystemBackground
        card.layer.cornerRadius = 16

        let label = UILabel()
        label.text = text
        label.font = .systemFont(ofSize: 15)
        label.textColor = .secondaryLabel
        label.numberOfLines = 0
        label.textAlignment = .center
        label.translatesAutoresizingMaskIntoConstraints = false

        card.addSubview(label)
        NSLayoutConstraint.activate([
            label.topAnchor.constraint(equalTo: card.topAnchor, constant: 16),
            label.bottomAnchor.constraint(equalTo: card.bottomAnchor, constant: -16),
            label.leadingAnchor.constraint(equalTo: card.leadingAnchor, constant: 20),
            label.trailingAnchor.constraint(equalTo: card.trailingAnchor, constant: -20),
        ])
        return card
    }
}
