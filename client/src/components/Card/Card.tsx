import styles from "./card.module.css"
import { LuUser } from "react-icons/lu"

interface CardProps {
	title: String
	sub: String
}

const Card = ({ title, sub }: CardProps) => {
	return (
		<div className={styles.card}>
			<h2
				className="heading"
				style={{
					letterSpacing: "-1px",
					display: "flex",
					flexDirection: "row",
					alignItems: "center",
				}}
			>
				<LuUser style={{ marginRight: "10px" }} />
				{title}
			</h2>
			<div className="sub" style={{ marginTop: "5px", color: "grey" }}>
				<p>{sub}</p>
			</div>
		</div>
	)
}

export default Card
